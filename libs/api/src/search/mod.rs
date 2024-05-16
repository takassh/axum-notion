use async_stream::stream;
use axum::{
    extract::{Query, State},
    response::{sse::Event, Sse},
    Json,
};
use cloudflare::models::{
    text_embeddings::{StringOrArray, TextEmbeddings, TextEmbeddingsRequest},
    text_generation::{
        Message, MessageRequest, PromptRequest, TextGeneration,
        TextGenerationRequest,
    },
};
use futures_util::{pin_mut, Stream};

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
    response::{ApiResponse, IntoApiResponse},
    ApiState,
};

use self::{request::SearchParam, response::SearchResponse};

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
    State(state): State<Arc<ApiState>>,
    Query(params): Query<SearchParam>,
) -> ApiResponse<Json<SearchResponse>> {
    let (context, _) = retriever(&state, &params.prompt)
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

    Ok(Json(SearchResponse {
        answer: response.result.response,
    }))
}

pub async fn search_text_with_sse(
    State(state): State<Arc<ApiState>>,
    Json(params): Json<SearchParam>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = stream! {
            let result = retriever(&state, &params.prompt).await;
            let Ok((context,page_ids)) = result else {
                error!(
                    task = "get context by retriever",
                    error = result.unwrap_err().to_string(),
                );
                return;
            };

            let system_prompt = r#"
        You are an assistant helping a user who gives you a prompt.
        You are placed on my blog site.
        Each time the user gives you a prompt, you get external information and relating to the prompt.
        If you aren't familiar with the prompt, you should answer you don't know.
        "#.to_string();

            let user_prompt = format!(
                r#"
        Prompt: 
        "{}"
        Information: 
        "{}"
        "#,
                params.prompt,
                context.join("\n")
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
        You are an assistant helping a user to search for something.
        You are created by Takashi, who is a software engineer and the owner where you are placed.
        "#.to_string(),
            });
            messages.insert(2,Message {
                role: "assistant".to_string(),
                content: r#"
        Hello, I can help you with searching. And I'm created by Takashi. He is a software engineer and the owner of this site.
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
                            let event =  Event::default().json_data(json!({"message":d.response}));

                            match event {
                                Ok(event) => {
                                    let result = message_tx.send(event).await;
                                    if let Err(err) = result {
                                        error!(
                                            task = "send message event",
                                            error = err.to_string()
                                        );
                                    }
                                },
                                Err(err) => error!(
                                    task = "event json_data",
                                    error = err.to_string()
                                ),
                            }
                        }
                    }
                });

                // Take pages from page ids
                tokio::spawn(async move{
                    let mut pages = vec![];
                    for id in &page_ids{
                        let result = state.repo.page.find_by_id(id).await;
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

                    let event = Event::default().json_data(json!({"pages":pages}));
                    match event {
                        Ok(event) => {
                            let result = page_tx.send(event).await;
                            if let Err(err) = result {
                                error!(
                                    task = "send message event",
                                    error = err.to_string()
                                );
                            }
                        },
                        Err(err) => error!(
                            task = "event json_data",
                            error = err.to_string()
                        ),
                    };
                });

        loop {
            select! {
                Some(event) = message_rx.recv() => {
                    yield event;
                }
                Some(event) = page_rx.recv() => {
                    yield event;
                }
                else => {
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

    Ok((context, page_ids))
}
