use crate::agent::function_call::Agent;
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
use notion_client::objects::{
    block::Block,
    page::{Page, PageProperty},
};
use rpc_router::CallResponse;
use tracing::info;

use qdrant_client::qdrant::{
    with_payload_selector::SelectorOptions, PayloadIncludeSelector,
    SearchPoints, WithPayloadSelector,
};
use serde_json::json;
use std::{collections::HashMap, convert::Infallible, sync::Arc};
use tokio::{select, sync::mpsc};
use tokio_stream::StreamExt as _;
use tracing::error;

use crate::{
    agent::function_call::{
        Function, FunctionCallAgent, Parameters, PropertyType, Tool,
    },
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
        let blocks = state.repo.block.find_by_notion_page_id("74c5e456-0feb-4049-a217-ba6ad67869ca").await;
        let Ok(Some(blocks)) = blocks else {
            error!(
                task = "get about page",
                error = blocks.unwrap_err().to_string(),
            );
            return;
        };


        let result = retriever(&state, &params.prompt).await;
        let Ok((context,mut page_ids)) = result else {
            error!(
                task = "get context by retriever",
                error = result.unwrap_err().to_string(),
            );
            return;
        };

        let blocks:Vec<Block> = serde_json::from_str(&blocks.contents).unwrap();
        let content_in_about = blocks.iter().flat_map(|block|{
            block.block_type.plain_text()
        }).flatten().collect::<Vec<_>>().join("\n");

        let title_and_dates = get_page_title_and_dates(&state).await;
        let Ok(title_and_dates) = title_and_dates else {
            error!(
                task = "get page title and dates",
                error = title_and_dates.unwrap_err().to_string(),
            );
            return;
        };

        let function_call_agent = FunctionCallAgent::new(
            state.cloudflare.clone(),
            vec![
            Tool {
                r#type: "function".to_string(),
                function: Function {
                    name: "find_article_by_word".to_string(),
                    description: "Retrieve articles with titles containing a specified word. The shorter the word, the more likely the result will be retrieved. This function may be called any number of times.".to_string(),
                    parameters: Some(
                        Parameters {
                            r#type: "object".to_string(),
                           properties:HashMap::from([
                            ("word".to_string(), PropertyType::String),
                        ]),
                            ..Default::default()
                        }
                    ),
                },
            },
        ],
        "What is this site?".to_string(),
        r#"<tool_call>{"arguments": {"word": "about"}, "name": "find_article_by_word"}</tool_call><tool_call>{"arguments": {"word": "site"}, "name": "find_article_by_word"}</tool_call>"#.to_string(),
        format!(r#"<tool_response>{{"title": "About", "content": "{}"}}</tool_response>"#,content_in_about),
        params.history.clone(),
    );

        let tool_calls = function_call_agent.prompt(&params.prompt).await;
        let Ok(tool_calls) = tool_calls else {
            error!(
                task = "function call prompt",
                error = tool_calls.unwrap_err().to_string(),
            );
            return;
        };


        let mut resources = vec![];
        for tool_call in tool_calls.clone(){
            let params = json!(&tool_call.arguments);
            let response = state.rpc.call_route(None,tool_call.name,Some(params)).await;
            let Ok(CallResponse { id: _, method: _, value }) = response else{
                error!(
                    task = "call route",
                    error = response.unwrap_err().to_string(),
                );
                continue;
            };

            let block = serde_json::from_value::<Option<entity::block::Block>>(value.clone());
            let Ok(block) = block else{
                error!(
                    task = "parse block",
                    value = value.to_string(),
                    error = block.unwrap_err().to_string(),
                );
                continue;
            };
            let Some(entity::block::Block{ notion_page_id, updated_at:_, contents }) = block else{
                continue;
            };

            let block = serde_json::from_str::<Vec<Block>>(&contents);
            let Ok(block) = block else{
                error!(
                    task = "parse block",
                    contents = contents,
                    error = block.unwrap_err().to_string(),
                );
                continue;
            };
            resources.push(block.iter().flat_map(|b|b.block_type.plain_text()).flatten().collect::<Vec<_>>().join("\n"));
            page_ids.push(notion_page_id);
        }

        info!(
            task = "tool calls",
            prompt = &params.prompt,
            tool_calls = tool_calls.into_iter().map(|t|format!("function:{},arguments:{:?}",t.name,t.arguments)).collect::<Vec<_>>().join("\n"),
            page_ids = &page_ids.join(","),
            resources = &resources.join("\n"),
        );

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
        Resources:
        "{}"
        Current Date:
        "{}"
        "#,
                params.prompt,
                context.join("\n"),
                resources.join("\n"),
                chrono::Utc::now().format("%d/%m/%Y %H:%M")
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
        page_ids = &page_ids.join(","),
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
