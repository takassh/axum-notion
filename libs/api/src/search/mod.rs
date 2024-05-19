use async_stream::stream;
use axum::{
    extract::{Query, State},
    response::{sse::Event, Sse},
    Extension, Json,
};
use cloudflare::models::{
    text_embeddings::{StringOrArray, TextEmbeddings, TextEmbeddingsRequest},
    text_generation::{
        Message, MessageRequest, PromptRequest, TextGeneration,
        TextGenerationRequest,
    },
};
use entity::prelude::*;
use futures_util::{pin_mut, Stream};
use notion_client::objects::page::{Page, PageProperty};
use tracing::info;

use qdrant_client::qdrant::{
    with_payload_selector::SelectorOptions, PayloadIncludeSelector,
    SearchPoints, WithPayloadSelector,
};
use serde_json::json;
use std::{convert::Infallible, sync::Arc};
use tokio::{select, sync::mpsc};
use tokio_stream::StreamExt as _;
use tracing::error;

use crate::{
    auth::Claims,
    response::{ApiResponse, IntoApiResponse},
    ApiState,
};

use self::{request::SearchParam, response::SearchResp};

pub mod request;
pub mod response;

/// Search with prompt
#[utoipa::path(
    get,
    path = "/search",
    responses(
        (status = 200, description = "Search with prompt successfully", body = [SearchResponse])
    ),
    params(
        SearchParam
    )
)]
pub async fn search_text(
    Extension(ref claims): Extension<Claims>,
    State(state): State<Arc<ApiState>>,
    Query(params): Query<SearchParam>,
) -> ApiResponse<Json<SearchResp>> {
    let (context, page_ids) = retriever(&state, &params.prompt)
        .await
        .into_response("502-012")?;

    let prompt = format!(
        r#"
        You are an assistant helping a user to search for something.
        The user provides a prompt "{}" and you need to generate a response based on given contexts.
        If the context doesn't make sense with the prompt, you should answer you don't know.
        Your answer must be concise.

        Context:
        "{}"
        
        Answer:
        "#,
        params.prompt,
        context.join("\n")
    );

    let response = state
        .cloudflare
        .llama_3_8b_instruct(TextGenerationRequest::Prompt(PromptRequest {
            prompt: prompt.to_string(),
            ..Default::default()
        }))
        .await
        .into_response("502-014")?;

    let session = save_prompt(
        &state,
        params.session,
        claims.user_id.unwrap(),
        &params.prompt,
        &response.result.response,
        page_ids,
    )
    .await
    .into_response("502-019")?;

    Ok(Json(SearchResp {
        answer: response.result.response,
        session,
    }))
}

pub async fn search_text_with_sse(
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<ApiState>>,
    Json(params): Json<SearchParam>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = stream! {
        let title_and_dates = get_page_title_and_dates(&state).await;
        let Ok(title_and_dates) = title_and_dates else {
            error!(
                task = "get page title and dates",
                error = title_and_dates.unwrap_err().to_string(),
            );
            return;
        };
            let result = retriever(&state, &params.prompt).await;
            let Ok((context,page_ids)) = result else {
                error!(
                    task = "get context by retriever",
                    error = result.unwrap_err().to_string(),
                );
                return;
            };

            let system_prompt = format!(r#"
        You are an assistant helping a user who gives you a prompt.
        You are placed on my blog site.
        Each time the user gives you a prompt, you get external information relating to the prompt and current date.
        If you aren't familiar with the prompt, you should answer you don't know.
        Here are title and created time of all articles in the site:
        {}
        "#,title_and_dates.iter().map(|(title,date)|format!("{},{}",title,date)).collect::<Vec<_>>().join("\n"));

            let user_prompt = format!(
                r#"
        Prompt: 
        "{}"
        Information: 
        "{}"
        Current Date:
        "{}"
        "#,
                params.prompt,
                context.join("\n"),
                chrono::Utc::now().format("%d/%m/%Y %H:%M").to_string()
            );

            let mut messages = params.history;
            messages.insert(0,Message {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            });
            messages.insert(1,Message {
                role: "user".to_string(),
                content: r#"
        Prompt: 
        "Hello, What can you help me?"
        Information: 
        You are an assistant helping a user.
        You are created by Takashi, who is a software engineer and the owner where you are placed.
        Your name is Takashi AI.
        "#.to_string(),
            });
            messages.insert(2,Message {
                role: "assistant".to_string(),
                content: r#"
        Hello, My name is Takashi AI. I'm created by Takashi. He is a software engineer and the owner of this site. What can I help you with?
        "#.to_string(),
            });
            messages.push(Message {
                role: "user".to_string(),
                content: user_prompt.to_string(),
            });



                let (message_tx, mut message_rx) = mpsc::channel(100);
                let (page_tx, mut page_rx) = mpsc::channel(1);

                // Receive messages from LLM
                let _state = state.clone();
                tokio::spawn(async move{
                    let response = _state
                    .cloudflare
                    .llama_3_8b_instruct_with_stream(TextGenerationRequest::Message(
                        MessageRequest {
                            messages,
                            stream: Some(true),
                            ..Default::default()
                        },
                    ));
                    pin_mut!(response); // needed for iteration
                    while let Ok(Some(data)) = response.next().await.transpose() {
                        for d in data{
                            let result = message_tx.send(d.response).await;
                                              if let Err(err) = result {
                                        error!(
                                            task = "send message event",
                                            error = err.to_string()
                                        );
                                    }

                        }
                    }
                });

                // Take pages from page ids
                let _state = state.clone();
                tokio::spawn(async move{
                    let mut pages = vec![];
                    for id in &page_ids{
                        let result = _state.repo.page.find_by_id(id).await;
                        let Ok(page) = result else {
                            error!(
                                task = "get page by notion client",
                                error = result.unwrap_err().to_string(),
                            );
                            continue;
                        };

                        let Some(page) = page else{
                            continue;
                        };

                        pages.push(page.contents);
                    }

                    let result = page_tx.send(pages).await;
                    if let Err(err) = result {
                        error!(
                            task = "send message event",
                            error = err.to_string()
                        );
                    }
                });

                let mut all_messages = String::new();
                let mut page_ids = vec![];
        loop {
            select! {
                Some(message) = message_rx.recv() => {
                    let event =  Event::default().json_data(json!({"message":message}));
                     let Ok(event) = event else {
                        error!(
                            task = "event json_data",
                            error = event.unwrap_err().to_string()
                        );
                        continue;
                    };

                    all_messages.push_str(&message);
                    yield event;
                }
                Some(pages) = page_rx.recv() => {
                    page_ids = pages.clone().iter().map(|s|serde_json::from_str::<Page>(s).unwrap().id).collect();
                    let event =  Event::default().json_data(json!({"pages":pages}));
                    let Ok(event) = event else {
                       error!(
                           task = "event json_data",
                           error = event.unwrap_err().to_string()
                       );
                       continue;
                   };
                    yield event;
                }
                else => {
                    let session = save_prompt(
                        &state,
                        params.session,
                        claims.user_id.unwrap(),
                        &params.prompt,
                        &all_messages,
                        page_ids,
                    ).await;
                    let Ok(session) = session else {
                        error!(
                            task = "save prompt",
                            error = session.unwrap_err().to_string(),
                        );
                        break;
                    };
                    let event =  Event::default().json_data(json!({"session":session}));
                    let Ok(event) = event else {
                       error!(
                           task = "event json_data",
                           error = event.unwrap_err().to_string()
                       );
                       break;
                    };

                    yield event;
                    break;
                }
            }
        };
    };

    Sse::new(stream.map(Ok))
}

async fn retriever(
    state: &Arc<ApiState>,
    prompt: &str,
) -> anyhow::Result<(Vec<String>, Vec<String>)> {
    let embedding = state
        .cloudflare
        .bge_small_en_v1_5(TextEmbeddingsRequest {
            text: StringOrArray::from(prompt),
        })
        .await?;

    let Some(vector) = embedding.result.data.first() else {
        return Err(anyhow::anyhow!("No vectors found"));
    };

    let search_result = state
        .qdrant
        .search_points(&SearchPoints {
            collection_name: state.config.qdrant.collection.clone(),
            vector: vector.clone(),
            limit: 5,
            with_payload: Some(WithPayloadSelector {
                selector_options: Some(SelectorOptions::Include(
                    PayloadIncludeSelector {
                        fields: vec![
                            "document".to_string(),
                            "page_id".to_string(),
                        ],
                    },
                )),
            }),
            ..Default::default()
        })
        .await?;

    let mut context: Vec<String> = vec![];
    let mut page_ids: Vec<String> = vec![];
    for result in search_result.result.iter() {
        if result.score < 0.6 {
            continue;
        }

        // Take context
        let Some(doc) = result.payload.get("document") else {
            continue;
        };
        let Some(doc) = doc.as_str() else {
            continue;
        };

        context.push(doc.to_string());

        // Take page id
        let Some(page_id) = result.payload.get("page_id") else {
            continue;
        };
        let Some(page_id) = page_id.as_str() else {
            continue;
        };
        page_ids.push(page_id.to_string());
    }

    info!(
        task = "retriever",
        prompt = prompt,
        context = context.join("\n"),
    );

    Ok((context, page_ids))
}

async fn save_prompt(
    state: &Arc<ApiState>,
    session_id: Option<String>,
    user_id: i32,
    prompt: &str,
    answer: &str,
    page_ids: Vec<String>,
) -> anyhow::Result<String> {
    let prompt_session_id = state
        .repo
        .prompt_session
        .save(PromptSessionEntity {
            id: session_id.unwrap_or_default(),
            user_id,
            ..Default::default()
        })
        .await?;

    state
        .repo
        .prompt
        .save(
            PromptEntity {
                prompt_session_id: prompt_session_id.clone(),
                user_prompt: prompt.to_string(),
                assistant_prompt: answer.to_string(),
                ..Default::default()
            },
            page_ids,
        )
        .await?;

    Ok(prompt_session_id)
}

async fn get_page_title_and_dates(
    state: &Arc<ApiState>,
) -> anyhow::Result<Vec<(String, String)>> {
    let pages = state.repo.page.find_all().await?;
    Ok(pages
        .iter()
        .flat_map(|page| {
            let page = serde_json::from_str::<Page>(&page.contents)?;
            let title_and_date =
                if let Some(PageProperty::Title { id: _, title }) =
                    page.properties.get("title")
                {
                    let title = title
                        .iter()
                        .flat_map(|t| t.plain_text())
                        .collect::<Vec<_>>()
                        .join("");
                    let date =
                        page.created_time.format("%d/%m/%Y %H:%M").to_string();

                    (title, date)
                } else {
                    return Err(anyhow::anyhow!("No title found"));
                };

            Ok(title_and_date)
        })
        .collect())
}
