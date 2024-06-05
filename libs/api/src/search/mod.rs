use crate::agent::{question_and_answer::QuestionAnswerAgent, Agent};
use async_stream::stream;
use axum::{
    extract::{Query, State},
    response::{sse::Event, Sse},
    Extension, Json,
};
use cloudflare::models::{
    text_embeddings::{StringOrArray, TextEmbeddings, TextEmbeddingsRequest},
    text_generation::{PromptRequest, TextGeneration, TextGenerationRequest},
};
use entity::prelude::*;
use futures_util::Stream;
use notion_client::objects::page::{Page, PageProperty};
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
        None,
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
        let result = retriever(&state, &params.prompt).await;
        let Ok((result,mut page_ids)) = result else {
            error!(
                task = "get context by retriever",
                error = result.unwrap_err().to_string(),
            );
            return;
        };

        let mut vector_result = String::new();
        if !result.is_empty(){
            vector_result = format!("## Vector search result\n{}",result.join("\n"))
        }

        let function_call_agent = FunctionCallAgent::new(
            state.cloudflare.clone(),
            vec![
            Tool {
                r#type: "function".to_string(),
                function: Function {
                    name: "find_article_by_word".to_string(),
                    description: "Retrieve articles with titles containing a specified word. Feel free to call it when you search an article".to_string(),
                    parameters: Some(
                        Parameters {
                            r#type: "object".to_string(),
                           properties:HashMap::from([
                            ("word".to_string(), PropertyType::String),
                        ]),
                          required: Some(vec!["word".to_string()]),
                        }
                    ),
                },
            },
            Tool {
                r#type: "function".to_string(),
                function: Function {
                    name: "get_current_datetime".to_string(),
                    description: "Get current datetime".to_string(),
                    parameters: Some(
                        Parameters {
                            r#type: "object".to_string(),
                           properties:HashMap::from([
                            ("timezone".to_string(), PropertyType::String),
                        ]),
                          required: None,
                        }
                    ),
                },
            },
            Tool {
                r#type: "function".to_string(),
                function: Function {
                    name: "get_articles_with_date".to_string(),
                    description: "Get all having articles with created time which this blog site has. Feel free to call it when you introduce articles.".to_string(),
                    parameters: Some(
                        Parameters {
                            r#type: "object".to_string(),
                           properties:HashMap::from([
                            ("limit".to_string(), PropertyType::String),
                        ]),
                          required: None,
                        }
                    ),
                },
            },
        ],
        params.history.clone(),
    );

        let tool_calls = function_call_agent.prompt(&params.prompt,None).await;
        let Ok(tool_calls) = tool_calls else {
            error!(
                task = "function call prompt",
                error = tool_calls.unwrap_err().to_string(),
            );
            return;
        };


        let mut function_result = vec![];
        for tool_call in tool_calls.clone(){
            let params = json!(&tool_call.arguments);
            let response = state.rpc.call_route(None,tool_call.name,Some(params)).await;
            let Ok(CallResponse { id: _, method, value }) = response else{
                error!(
                    task = "call route",
                    error = response.unwrap_err().to_string(),
                );
                continue;
            };

            match method.as_str(){
                "find_article_by_word" => {
                    let page = serde_json::from_value::<Option<entity::page::Page>>(value.clone());
                    let Ok(page) = page else{
                        error!(
                            task = "parse page",
                            value = value.to_string(),
                            error = page.unwrap_err().to_string(),
                        );
                        continue;
                    };
                    let Some(entity::page::Page{notion_page_id,updated_at:_,contents, notion_parent_id:_, parent_type:_, created_at:_, title, draft:_ }) = page else{
                        continue;
                    };

                    let page = serde_json::from_str::<Page>(&contents);
                    let Ok(page) = page else{
                        error!(
                            task = "parse page",
                            contents = contents,
                            error = page.unwrap_err().to_string(),
                        );
                        continue;
                    };

                    let Some(summary) = page.properties.get("summary") else {
                        continue;
                    };
                    let PageProperty::RichText { id: _, rich_text } = summary else {
                        error!(
                            task = "failed to get summary",
                            contents = contents,
                        );
                        continue;
                    };

                    let summary = rich_text
                        .iter()
                        .flat_map(|t| t.plain_text())
                        .collect::<Vec<_>>()
                        .join("");

                        function_result.push(format!("## Article title and summary\n### title\n{}\n### summary\n{}",title,summary));
                            page_ids.push(notion_page_id);
                }
                "get_current_datetime" => {
                    function_result.push(format!("## Current datetime\n{}",value));
                }
                "get_articles_with_date" => {
                    let titles_with_date = serde_json::from_value::<Vec<(String,String)>>(value.clone());
                    let Ok(titles_with_date) = titles_with_date else{
                        error!(
                            task = "parse get_articles_with_date",
                            value = value.to_string(),
                            error = titles_with_date.unwrap_err().to_string(),
                        );
                        continue;
                    };
                    let result = titles_with_date.iter().map(|(title,date)|format!("- {}, created at {}",title,date)).collect::<Vec<_>>().join("\n");
                    function_result.push(format!("## All article titles\n{}",result));
                }
                _ => {}
            }
        }

        let tool_calls = tool_calls.into_iter().map(|t|format!("function:{},arguments:{:?}",t.name,t.arguments)).collect::<Vec<_>>().join("\n");
        info!(
            task = "tool calls",
            prompt = &params.prompt,
            tool_calls = tool_calls,
            page_ids = &page_ids.join(","),
            resources = &function_result.join("\n"),
        );

        let (message_tx, mut message_rx) = mpsc::channel(100);
        let (page_tx, mut page_rx) = mpsc::channel(1);

        let question_answer_agent = QuestionAnswerAgent::new(
            state.cloudflare.clone(),
            params.history.clone(),
            );


        let _prompt = params.prompt.clone();
        let context = format!("{}\n{}",vector_result,function_result.join("\n"));
        let _context = context.clone();
        tokio::spawn(async move{
            let mut question_answer = question_answer_agent.prompt(&_prompt,Some(&_context)).await;
        while let Ok(Some(data)) = question_answer.next().await.transpose() {
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

                let contents = page.contents;
                let page = serde_json::from_str::<Page>(&contents);
                let Ok(page) = page else{
                    error!(
                        task = "parse page",
                        contents = contents,
                        error = page.unwrap_err().to_string(),
                    );
                    continue;
                };
                pages.push(page);
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
                            task = "event json_data message",
                            error = event.unwrap_err().to_string()
                        );
                        continue;
                    };

                    all_messages.push_str(&message);
                    yield event;
                }
                Some(pages) = page_rx.recv() => {
                    page_ids = pages.iter().map(|p|p.id.clone()).collect();
                    let event =  Event::default().json_data(json!({"pages":pages}));
                    let Ok(event) = event else {
                       error!(
                           task = "event json_data pages",
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
                        Some(tool_calls.as_str()),
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
                           task = "event json_data session",
                           error = event.unwrap_err().to_string()
                       );
                       break;
                    };

                    yield event;

                    let event =  Event::default().json_data(json!({"debug": {"context":&context}}));
                    let Ok(event) = event else {
                       error!(
                           task = "event json_data debug",
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
    tools: Option<&str>,
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
                tools_prompt: tools.unwrap_or_default().to_string(),
                ..Default::default()
            },
            page_ids,
        )
        .await?;

    Ok(prompt_session_id)
}
