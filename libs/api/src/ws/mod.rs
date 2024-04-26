use std::sync::Arc;

use anyhow::anyhow;
use anyhow::Context;
use aws_sdk_s3::primitives::ByteStream;
use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
};

use axum_extra::headers;
use axum_extra::TypedHeader;
use entity::prelude::*;
use futures_util::{SinkExt, StreamExt};
use notion_client::{
    endpoints::pages::update::request::UpdatePagePropertiesRequest,
    objects::{
        file::{ExternalFile, File},
        parent::Parent,
    },
};

use serde::Deserialize;
use serde::Serialize;
use tokio::sync::Mutex;

use tracing::{error, info};

use crate::ACCEPT_API_KEY;
use crate::{page::request::GenerateCoverImageRequest, ApiState};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "action")]
enum Action {
    Auth {
        api_key: String,
    },
    GenerateCoverImage {
        id: String,
        body: GenerateCoverImageRequest,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "action")]
enum ActionResponse {
    Auth { message: String },
    GenerateCoverImage { message: String },
}

pub async fn ws(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    State(state): State<ApiState>,
) -> Response {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    info!("`{user_agent}` connected.");
    ws.on_upgrade(|socket| handler(socket, state))
}

async fn handler(socket: WebSocket, state: ApiState) {
    info!("Connection opened");

    let authorized = Arc::new(Mutex::new(false));
    let (mut sender, mut receiver) = socket.split();

    loop {
        let msg = receiver.next().await;
        let Some(Ok(msg)) = msg else {
            error!(
                task = "receive message",
                error = msg.unwrap().unwrap_err().to_string()
            );
            return;
        };

        match msg {
            Message::Close(_) => {
                info!("Connection closed");
                return;
            }
            Message::Text(text) => {
                let text = serde_json::from_str::<Action>(&text);
                match text {
                    Ok(Action::Auth { api_key }) => {
                        let mut authorized = authorized.lock().await;
                        *authorized = authorize(&api_key);

                        let message = if *authorized {
                            "authenticated".to_string()
                        } else {
                            "invalid token".to_string()
                        };

                        let response =
                            serde_json::to_string(&ActionResponse::Auth {
                                message,
                            })
                            .unwrap();

                        let _ = sender.send(Message::Text(response)).await;
                    }
                    Ok(Action::GenerateCoverImage { id, body }) => {
                        {
                            if !*authorized.lock().await {
                                let _ = sender
                                    .send(Message::Text(serde_json::to_string(
                                        &ActionResponse::GenerateCoverImage {
                                            message: "not allowed".to_string(),
                                        },
                                    ).unwrap()))
                                    .await;
                                continue;
                            }
                        }

                        if id.is_empty() {
                            let _ = sender
                                .send(Message::Text(
                                    serde_json::to_string(
                                        &ActionResponse::GenerateCoverImage {
                                            message: "invalid id".to_string(),
                                        },
                                    )
                                    .unwrap(),
                                ))
                                .await;
                            continue;
                        }

                        if body.prompt.is_empty() {
                            let _ = sender
                                .send(Message::Text(
                                    serde_json::to_string(
                                        &ActionResponse::GenerateCoverImage {
                                            message: "invalid prompt"
                                                .to_string(),
                                        },
                                    )
                                    .unwrap(),
                                ))
                                .await;
                            continue;
                        }

                        let result =
                            generate_cover_image(&state, id, body).await;

                        let message = match result {
                            Ok(_) => "success".to_string(),
                            Err(e) => e.to_string(),
                        };

                        let response = serde_json::to_string(
                            &ActionResponse::GenerateCoverImage { message },
                        )
                        .unwrap();

                        let _ = sender.send(Message::Text(response)).await;
                    }
                    _ => {
                        let _ = sender
                            .send(Message::Text("invalid method".to_string()))
                            .await;
                    }
                }
            }
            _ => {}
        }
    }
}

fn authorize(api_key: &str) -> bool {
    let accept_api_key = ACCEPT_API_KEY.get().unwrap();
    api_key == accept_api_key
}

pub async fn generate_cover_image(
    state: &ApiState,
    id: String,
    body: GenerateCoverImageRequest,
) -> anyhow::Result<()> {
    let response = state
        .cloudflare
        .post(
            state.config.cloudflare.generate_ai_path.as_str(),
            serde_json::to_string(&body).context("failed to serialize body")?,
        )
        .await?;

    let file_name = format!("{}.png", id);

    let image = response
        .bytes()
        .await
        .context("failed to get response bytes")?;

    state
        .s3
        .put_object()
        .bucket(state.config.aws.bucket.clone())
        .content_type("image/png")
        .key(file_name.clone())
        .body(ByteStream::from(image))
        .send()
        .await
        .context("failed to put object")?;

    state
        .notion
        .pages
        .update_page_properties(
            &id,
            UpdatePagePropertiesRequest {
                cover: Some(File::External {
                    external: ExternalFile {
                        url: format!(
                            "{}/{}?t={}",
                            state.config.aws.s3_url,
                            file_name,
                            chrono::Utc::now().timestamp()
                        ),
                    },
                }),
                ..Default::default()
            },
        )
        .await
        .context("failed to update page properties")?;

    let page = state
        .notion
        .pages
        .retrieve_a_page(&id, None)
        .await
        .context("failed to retrieve a page")?;

    let json = serde_json::to_string_pretty(&page)
        .context("failed to serialize page")?;
    let parent_id = match page.parent {
        Parent::DatabaseId { database_id } => database_id,
        _ => Err(anyhow!("parent is not database id"))?,
    };
    let model = PageEntity {
        notion_page_id: page.id,
        notion_database_id: parent_id,
        contents: json,
        created_at: page.created_time,
        ..Default::default()
    };

    state
        .repo
        .page
        .save(model)
        .await
        .context("failed to save page")?;

    Ok(())
}
