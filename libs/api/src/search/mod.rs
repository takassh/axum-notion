use crate::agent::{
    get_template, question_and_answer::QuestionAnswerAgent, Agent,
};
use async_stream::stream;
use axum::{
    extract::State,
    response::{sse::Event, Sse},
    Extension, Json,
};
use cloudflare::models::{
    text_embeddings::{StringOrArray, TextEmbeddings, TextEmbeddingsRequest},
    text_generation::{
        Function, ModelParameters, Parameters, PropertyType,
        TextGenerationJsonResult, Tool, ToolCall, LLAMA_3_8B_INSTRUCT,
    },
};
use entity::prelude::*;
use futures_util::Stream;
use langfuse::{
    apis::ingestion_api::ingestion_batch,
    models::{
        ingestion_event_one_of, ingestion_event_one_of_2,
        ingestion_event_one_of_4, CreateGenerationBody, CreateSpanBody,
        IngestionBatchRequest, IngestionEvent, IngestionEventOneOf,
        IngestionEventOneOf2, IngestionEventOneOf4, TraceBody,
    },
};
use notion_client::objects::{
    block::Block,
    page::{Page, PageProperty},
};
use qdrant_client::qdrant::{
    condition::ConditionOneOf, r#match::MatchValue,
    with_payload_selector::SelectorOptions, Condition, FieldCondition, Filter,
    Match, PayloadIncludeSelector, SearchPoints, WithPayloadSelector,
};
use rpc_router::CallResponse;
use tracing::info;
use uuid::Uuid;

use serde_json::json;
use std::{collections::HashMap, convert::Infallible, sync::Arc};
use tokio::{
    join, select,
    sync::{mpsc, oneshot},
};
use tokio_stream::StreamExt as _;
use tracing::error;

use crate::{agent::function_call::FunctionCallAgent, auth::Claims, ApiState};

use self::request::SearchParam;

pub mod request;

pub async fn search_text_with_sse(
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<ApiState>>,
    Json(params): Json<SearchParam>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = stream! {

        let result = generate_keyword(&params,&state).await;
        let Ok((keyword_response,keyword_log)) = result else {
            error!(
                task = "keyword",
                error = result.unwrap_err().to_string(),
            );
            return;
        };

        let keywords = keyword_response.response.clone().unwrap_or_default();

        let vector_search = vector_search(&state,&keywords.clone()).await;
        let Ok((vector_search_result,vector_page_ids,vector_log)) = vector_search else {
            error!(
                task = "vector search",
                error = vector_search.unwrap_err().to_string(),
            );
            return;
        };

        let result = function_call(&params,&state,&keywords.clone()).await;
        let Ok((function_call_response,observations,function_page_ids,function_call_log,observation_log)) = result else {
            error!(
                task = "function call",
                error = result.unwrap_err().to_string(),
            );
            return;
        };


        let all_page_ids = vector_page_ids.into_iter().chain(function_page_ids.into_iter()).collect::<Vec<_>>();

        let (message_tx, mut message_rx) = mpsc::channel(100);
        let (page_tx, mut page_rx) = mpsc::channel(1);
        let (qa_log_tx, qa_log_rx) = oneshot::channel();
        let context = format!(
            "{}\n{}",
            vector_search_result.join("\n"),
            observations.join("\n")
        );

        let _params = params.clone();
        let _context = context.clone();
        let _state = state.clone();
        tokio::spawn(async move {
           let (_,_) = join!(
                make_answer(&_params,&_state,&_context,message_tx,qa_log_tx),
                get_pages_by_ids(&_state,&all_page_ids,page_tx),
            );
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
                        &params,
                        claims.user_id.unwrap(),
                        &all_messages,
                        &function_call_response.tool_calls,
                        page_ids,
                    ).await;



                    let Ok(session) = session else {
                        error!(
                            task = "save prompt",
                            error = session.unwrap_err().to_string(),
                        );
                        break;
                    };

                    let event =  Event::default().json_data(json!({"session":&session}));
                    let Ok(event) = event else {
                       error!(
                           task = "event json_data session",
                           error = event.unwrap_err().to_string()
                       );
                       break;
                    };

                    yield event;

                    let trace_id = Uuid::new_v4().to_string();

                    let event =  Event::default().json_data(json!({"debug": {"context":&context, "traceId":trace_id.clone()}}));
                    let Ok(event) = event else {
                       error!(
                           task = "event json_data debug",
                           error = event.unwrap_err().to_string()
                       );
                       break;
                    };
                    yield event;


                    let qa_log = qa_log_rx.await;
                    let Ok(qa_log) = qa_log else {
                        error!(
                            task = "qa log",
                            error = qa_log.unwrap_err().to_string(),
                        );
                        break;
                    };
                    let log = log_langfuse(&claims,&params,&state,&session,trace_id,keyword_log,vector_log,function_call_log,observation_log,qa_log).await;

                    let Ok(_) = log else {
                        error!(
                            task = "ingestion",
                            error = log.unwrap_err().to_string(),
                        );
                        break;
                    };

                    break;
                }
            }
        };
    };

    Sse::new(stream.map(Ok))
}

async fn save_prompt(
    state: &Arc<ApiState>,
    params: &SearchParam,
    user_id: i32,
    answer: &str,
    tool_calls: &Option<Vec<ToolCall>>,
    page_ids: Vec<String>,
) -> anyhow::Result<String> {
    let mut tools_prompt = None;
    if let Some(tool_calls) = tool_calls {
        tools_prompt = Some(
            tool_calls
                .iter()
                .map(|t| {
                    format!("function:{},arguments:{:?}", t.name, t.arguments)
                })
                .collect::<Vec<_>>()
                .join("\n"),
        );
    }
    let prompt_session_id = state
        .repo
        .prompt_session
        .save(PromptSessionEntity {
            id: params.session.clone().unwrap_or_default(),
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
                user_prompt: params.prompt.to_string(),
                assistant_prompt: answer.to_string(),
                tools_prompt: tools_prompt.unwrap_or_default().to_string(),
                ..Default::default()
            },
            page_ids,
        )
        .await?;

    Ok(prompt_session_id)
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

    let page_search_result = state
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
            filter: Some(Filter {
                must: vec![Condition {
                    condition_one_of: Some(ConditionOneOf::Field(
                        FieldCondition {
                            key: "type".to_string(),
                            r#match: Some(Match {
                                match_value: Some(MatchValue::Keyword(
                                    serde_json::to_string(
                                        &DocumentTypeEntity::Page,
                                    )
                                    .unwrap(),
                                )),
                            }),
                            ..Default::default()
                        },
                    )),
                }],
                ..Default::default()
            }),
            ..Default::default()
        })
        .await?;

    let block_search_result = state
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
            filter: Some(Filter {
                must: vec![Condition {
                    condition_one_of: Some(ConditionOneOf::Field(
                        FieldCondition {
                            key: "type".to_string(),
                            r#match: Some(Match {
                                match_value: Some(MatchValue::Keyword(
                                    serde_json::to_string(
                                        &DocumentTypeEntity::Block,
                                    )
                                    .unwrap(),
                                )),
                            }),
                            ..Default::default()
                        },
                    )),
                }],
                ..Default::default()
            }),
            ..Default::default()
        })
        .await?;

    let search_result = page_search_result
        .result
        .into_iter()
        .chain(block_search_result.result.into_iter())
        .collect::<Vec<_>>();

    let mut context: Vec<String> = vec![];
    let mut page_ids: Vec<String> = vec![];
    for result in search_result.iter() {
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

async fn generate_keyword(
    params: &SearchParam,
    state: &Arc<ApiState>,
) -> anyhow::Result<(TextGenerationJsonResult, CreateGenerationBody)> {
    let system_prompt =
        get_template(&state.langfuse, "keyword-generator-system").await?;

    let keyword_generator = QuestionAnswerAgent::new(
        state.cloudflare.clone(),
        "keyword generator".to_string(),
        system_prompt,
        params.history.clone(),
        Some(ModelParameters {
            max_tokens: Some(20),
            ..Default::default()
        }),
    );

    let user_prompt_template =
        get_template(&state.langfuse, "keyword-generator-user").await?;

    let (keyword_result, log) = keyword_generator
        .prompt(&user_prompt_template, &params.prompt, None)
        .await?;

    Ok((keyword_result[0].clone(), log))
}

async fn vector_search(
    state: &Arc<ApiState>,
    keywords: &str,
) -> anyhow::Result<(Vec<String>, Vec<String>, CreateSpanBody)> {
    let mut vector_result = vec![];
    let mut all_page_ids = vec![];
    let keywords = keywords.split(',');

    let mut span = CreateSpanBody {
        id: Some(Some(Uuid::new_v4().to_string())),
        name: Some(Some("search vectors".to_string())),
        start_time: Some(Some(chrono::Utc::now().to_rfc3339())),
        end_time: Some(Some(chrono::Utc::now().to_rfc3339())),
        input: Some(Some(serde_json::Value::String(
            keywords.clone().collect::<Vec<_>>().join(","),
        ))),
        ..Default::default()
    };

    for keyword in keywords {
        if keyword.is_empty() {
            continue;
        }
        let result = retriever(state, keyword).await;
        let Ok((result, page_ids)) = result else {
            error!(
                task = "get context by retriever",
                error = result.unwrap_err().to_string(),
            );
            continue;
        };

        if !result.is_empty() {
            let mut _result = vec![];
            let mut _page_ids = vec![];
            for (i, page_id) in page_ids.iter().enumerate() {
                if _page_ids.contains(page_id) {
                    continue;
                }
                _result.push(result[i].clone());
                _page_ids.push(page_ids[i].clone());
            }

            vector_result.push(format!(
                "## Vector search result with {}\n{}",
                keyword,
                _result
                    .iter()
                    .map(|c| format!("1. {}", c))
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
            all_page_ids.append(&mut _page_ids);
        } else {
            vector_result.push(format!(
                "## Vector search result with {}\nNot found",
                keyword
            ));
        }
    }

    span.end_time = Some(Some(chrono::Utc::now().to_rfc3339()));
    span.output = Some(Some(serde_json::Value::Array(
        vector_result
            .iter()
            .map(|t| {
                serde_json::json!({
                    "role": "observation",
                    "content": t,
                })
            })
            .collect(),
    )));
    span.metadata = Some(Some(serde_json::json!({
        "page_ids": all_page_ids,
    })));

    Ok((vector_result, all_page_ids, span))
}

#[allow(clippy::too_many_arguments)]
async fn function_call(
    params: &SearchParam,
    state: &Arc<ApiState>,
    keywords: &str,
) -> anyhow::Result<(
    TextGenerationJsonResult,
    Vec<String>,
    Vec<String>,
    CreateGenerationBody,
    CreateSpanBody,
)> {
    let function_call_agent = FunctionCallAgent::new(
        state.cloudflare.clone(),
        &state.langfuse,
        "function call".to_string(),
        vec![
        Tool {
            r#type: "function".to_string(),
            function: Function {
                name: "get_article_summary".to_string(),
                description: "Get an article summary which title is similar with a given word. Must be one word.".to_string(),
                parameters: Some(
                    Parameters {
                        r#type: "object".to_string(),
                       properties:HashMap::from([
                        ("query".to_string(), PropertyType::String),
                    ]),
                      required: Some(vec!["query".to_string()]),
                    }
                ),
            },
        },
        Tool {
            r#type: "function".to_string(),
            function: Function {
                name: "get_article_detail".to_string(),
                description: "Get an article detail which title is similar with a given word. Must be one word. This may be heavy because of full texts.".to_string(),
                parameters: Some(
                    Parameters {
                        r#type: "object".to_string(),
                       properties:HashMap::from([
                        ("query".to_string(), PropertyType::String),
                    ]),
                      required: Some(vec!["query".to_string()]),
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
                name: "get_article_title_list".to_string(),
                description: "Get article title list with created time.".to_string(),
                parameters: Some(
                    Parameters {
                        r#type: "object".to_string(),
                       properties:HashMap::from([
                        ("offset".to_string(), PropertyType::String),
                        ("limit".to_string(), PropertyType::String),
                    ]),
                    required: Some(vec!["offset".to_string(),"limit".to_string()]),
                    }
                ),
            },
        },
    ],
    params.history.clone(),
    Some(
        ModelParameters {
        temperature:Some(0),
        top_p:Some(0),
        top_k:Some(1),
         ..Default::default() }
        )).await?;

    let context = format!("## Possible search keywords\n{}", keywords);

    let user_prompt_template =
        get_template(&state.langfuse, "function-calls-user").await?;

    let (tool_calls_response, log) = function_call_agent
        .prompt(&user_prompt_template, &params.prompt, Some(&context))
        .await?;

    let mut span = CreateSpanBody {
        id: Some(Some(Uuid::new_v4().to_string())),
        name: Some(Some("get observations".to_string())),
        start_time: Some(Some(chrono::Utc::now().to_rfc3339())),
        end_time: Some(Some(chrono::Utc::now().to_rfc3339())),
        ..Default::default()
    };

    let Some(tool_calls) = &tool_calls_response.tool_calls else {
        return Ok((tool_calls_response, vec![], vec![], log, span));
    };

    let tool_calls = tool_calls.clone().into_iter().take(3);

    span.input = Some(Some(serde_json::Value::Array(
        tool_calls
            .clone()
            .map(|t| {
                serde_json::json!({
                    "role": "tool_call",
                    "content": t,
                })
            })
            .collect(),
    )));

    let mut observations = vec![];
    let mut page_ids = vec![];
    for tool_call in tool_calls.clone() {
        let params = json!(&tool_call.arguments);
        let response = state
            .rpc
            .call_route(None, tool_call.clone().name, Some(params.clone()))
            .await;
        let Ok(CallResponse {
            id: _,
            method,
            value,
        }) = response
        else {
            error!(
                task = "call route",
                error = response.unwrap_err().to_string(),
            );
            continue;
        };

        match method.as_str() {
            "get_article_summary" => {
                let page = serde_json::from_value::<Option<entity::page::Page>>(
                    value.clone(),
                );
                let Ok(page) = page else {
                    error!(
                        task = "parse page",
                        value = value.to_string(),
                        error = page.unwrap_err().to_string(),
                    );
                    continue;
                };
                let Some(entity::page::Page {
                    notion_page_id,
                    updated_at: _,
                    contents,
                    notion_parent_id: _,
                    parent_type: _,
                    created_at: _,
                    title,
                    draft: _,
                }) = page
                else {
                    observations.push(format!(
                        "## Article summary search results with {}\nNot found",
                        params.get("query").unwrap()
                    ));
                    continue;
                };

                let page = serde_json::from_str::<Page>(&contents);
                let Ok(page) = page else {
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
                let PageProperty::RichText { id: _, rich_text } = summary
                else {
                    error!(task = "failed to get summary", contents = contents,);
                    continue;
                };

                let summary = rich_text
                    .iter()
                    .flat_map(|t| t.plain_text())
                    .collect::<Vec<_>>()
                    .join("");

                observations.push(format!("## Article summary search results with {}\n### title\n{}\n### summary\n{}",params.get("query").unwrap(),title,summary));
                page_ids.push(notion_page_id);
            }
            "get_current_datetime" => {
                observations.push(format!("## Current datetime\n{}", value));
            }
            "get_article_detail" => {
                let block = serde_json::from_value::<
                    Option<entity::block::Block>,
                >(value.clone());
                let Ok(block) = block else {
                    error!(
                        task = "parse block entity",
                        value = value.to_string(),
                        error = block.unwrap_err().to_string(),
                    );
                    continue;
                };
                let Some(entity::block::Block {
                    notion_page_id,
                    updated_at: _,
                    contents,
                }) = block
                else {
                    observations.push(format!(
                        "## Article detail search results with {}\nNot found",
                        params.get("query").unwrap()
                    ));
                    continue;
                };

                let blocks = serde_json::from_str::<Vec<Block>>(&contents);
                let Ok(blocks) = blocks else {
                    error!(
                        task = "parse blocks",
                        contents = contents,
                        error = blocks.unwrap_err().to_string(),
                    );
                    continue;
                };

                let plain_text = blocks
                    .into_iter()
                    .flat_map(|b| b.block_type.plain_text())
                    .flatten()
                    .collect::<Vec<_>>()
                    .join("");

                observations.push(format!("## Article detail search results with {}\n### full texts\n{}",params.get("query").unwrap(),plain_text));
                page_ids.push(notion_page_id);
            }
            "get_article_title_list" => {
                let pages = serde_json::from_value::<Vec<entity::page::Page>>(
                    value.clone(),
                );
                let Ok(pages) = pages else {
                    error!(
                        task = "parse pages entity",
                        value = value.to_string(),
                        error = pages.unwrap_err().to_string(),
                    );
                    continue;
                };

                let mut titles_with_created_at = vec![];
                for page in pages {
                    let title = page.title;
                    let created_at = page.created_at;
                    titles_with_created_at.push(format!(
                        "- {}, created at {}",
                        title,
                        created_at.to_rfc3339()
                    ));
                }
                observations.push(format!(
                    "## Article title list with offset = {}, limit = {}\n{}",
                    params.get("offset").unwrap(),
                    params.get("limit").unwrap(),
                    titles_with_created_at.join("\n")
                ));
            }
            _ => {}
        }
    }

    span.end_time = Some(Some(chrono::Utc::now().to_rfc3339()));
    span.output = Some(Some(serde_json::Value::Array(
        observations
            .iter()
            .map(|t| {
                serde_json::json!({
                    "role": "observation",
                    "content": t,
                })
            })
            .collect(),
    )));

    Ok((tool_calls_response, observations, page_ids, log, span))
}

async fn make_answer(
    params: &SearchParam,
    state: &Arc<ApiState>,
    context: &str,
    message_tx: mpsc::Sender<String>,
    log_tx: oneshot::Sender<CreateGenerationBody>,
) -> anyhow::Result<()> {
    let system_prompt =
        get_template(&state.langfuse, "answer-generator-system").await?;
    let question_answer_agent = QuestionAnswerAgent::new(
        state.cloudflare.clone(),
        "answer generator".to_string(),
        system_prompt,
        params.history.clone(),
        None,
    );

    let prompt = &params.prompt;

    let user_prompt_template =
        get_template(&state.langfuse, "answer-generator-user").await?;

    let (qa_message, mut question_answer) = question_answer_agent
        .prompt_with_stream(&user_prompt_template, prompt, Some(context))
        .await;

    let mut log = CreateGenerationBody {
        id: Some(Some(Uuid::new_v4().to_string())),
        name: Some(Some("answer generator".to_string())),
        model: Some(Some(LLAMA_3_8B_INSTRUCT.to_string())),
        start_time: Some(Some(chrono::Utc::now().to_rfc3339())),
        input: Some(Some(serde_json::Value::Array(
            qa_message
                .iter()
                .map(|t| {
                    serde_json::json!({
                        "role": t.role,
                        "content": t.content,
                    })
                })
                .collect(),
        ))),
        ..Default::default()
    };

    let mut output = String::new();

    loop {
        let data = question_answer.next().await.transpose();
        let Ok(data) = data else {
            error!(
                task = "question answer",
                error = data.unwrap_err().to_string(),
            );
            break;
        };
        if let Some(data) = data {
            for d in data {
                if let Some(response) = d.response {
                    output.push_str(&response);
                    let result = message_tx.send(response).await;
                    if let Err(err) = result {
                        error!(
                            task = "send message event",
                            error = err.to_string()
                        );
                    }
                }
            }
        } else {
            // Finish answer
            break;
        }
    }

    log.output = Some(Some(serde_json::Value::String(output)));
    log.end_time = Some(Some(chrono::Utc::now().to_rfc3339()));
    let _ = log_tx.send(log);

    Ok(())
}

async fn get_pages_by_ids(
    state: &Arc<ApiState>,
    page_ids: &Vec<String>,
    page_tx: mpsc::Sender<Vec<Page>>,
) {
    let mut pages = vec![];
    for id in page_ids {
        let result = state.repo.page.find_by_id(id).await;
        let Ok(page) = result else {
            error!(
                task = "get page by notion client",
                error = result.unwrap_err().to_string(),
            );
            continue;
        };

        let Some(page) = page else {
            continue;
        };

        let contents = page.contents;
        let page = serde_json::from_str::<Page>(&contents);
        let Ok(page) = page else {
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
        error!(task = "send message event", error = err.to_string());
    }
}

#[allow(clippy::too_many_arguments)]
async fn log_langfuse(
    claims: &Claims,
    params: &SearchParam,
    state: &Arc<ApiState>,
    session_id: &str,
    trace_id: String,
    mut keyword_log: CreateGenerationBody,
    mut vector_log: CreateSpanBody,
    mut tool_calls_log: CreateGenerationBody,
    mut observation_log: CreateSpanBody,
    mut qa_log: CreateGenerationBody,
) -> anyhow::Result<()> {
    let env = state.env.clone();
    keyword_log.trace_id = Some(Some(trace_id.clone()));
    vector_log.trace_id = Some(Some(trace_id.clone()));
    tool_calls_log.trace_id = Some(Some(trace_id.clone()));
    observation_log.trace_id = Some(Some(trace_id.clone()));
    qa_log.trace_id = Some(Some(trace_id.clone()));
    let response = ingestion_batch(
        &state.langfuse,
        IngestionBatchRequest {
            batch: vec![
                // create trace
                IngestionEvent::IngestionEventOneOf(Box::new(
                    IngestionEventOneOf::new(
                        TraceBody {
                            id: Some(Some(trace_id.clone())),
                            timestamp: Some(Some(
                                chrono::Utc::now().to_rfc3339(),
                            )),
                            user_id: Some(Some(
                                claims.user_id.unwrap().to_string(),
                            )),
                            input: Some(Some(serde_json::Value::String(
                                params.prompt.clone(),
                            ))),
                            output: Some(Some(serde_json::Value::String(
                                qa_log
                                    .clone()
                                    .output
                                    .unwrap()
                                    .unwrap()
                                    .to_string(),
                            ))),
                            session_id: Some(Some(session_id.to_string())),
                            tags: Some(Some(vec![env])),
                            public: Some(Some(true)),
                            ..Default::default()
                        },
                        trace_id.clone(),
                        chrono::Utc::now().to_rfc3339(),
                        ingestion_event_one_of::Type::TraceCreate,
                    ),
                )),
                // keywords result
                IngestionEvent::IngestionEventOneOf4(Box::new(
                    IngestionEventOneOf4::new(
                        keyword_log,
                        Uuid::new_v4().to_string(),
                        chrono::Utc::now().to_rfc3339(),
                        ingestion_event_one_of_4::Type::GenerationCreate,
                    ),
                )),
                // keywords result
                IngestionEvent::IngestionEventOneOf2(Box::new(
                    IngestionEventOneOf2::new(
                        vector_log,
                        Uuid::new_v4().to_string(),
                        chrono::Utc::now().to_rfc3339(),
                        ingestion_event_one_of_2::Type::SpanCreate,
                    ),
                )),
                // function calls result
                IngestionEvent::IngestionEventOneOf4(Box::new(
                    IngestionEventOneOf4::new(
                        tool_calls_log,
                        Uuid::new_v4().to_string(),
                        chrono::Utc::now().to_rfc3339(),
                        ingestion_event_one_of_4::Type::GenerationCreate,
                    ),
                )),
                // observation result
                IngestionEvent::IngestionEventOneOf2(Box::new(
                    IngestionEventOneOf2::new(
                        observation_log,
                        Uuid::new_v4().to_string(),
                        chrono::Utc::now().to_rfc3339(),
                        ingestion_event_one_of_2::Type::SpanCreate,
                    ),
                )),
                // answer result
                IngestionEvent::IngestionEventOneOf4(Box::new(
                    IngestionEventOneOf4::new(
                        qa_log,
                        Uuid::new_v4().to_string(),
                        chrono::Utc::now().to_rfc3339(),
                        ingestion_event_one_of_4::Type::GenerationCreate,
                    ),
                )),
            ],
            metadata: None,
        },
    )
    .await;

    let Ok(response) = response else {
        error!(
            task = "ingestion",
            error = response.unwrap_err().to_string(),
        );
        return Err(anyhow::anyhow!("Failed to log langfuse"));
    };

    for error in response.errors {
        error!(task = "ingestion", error = error.message);
    }

    Ok(())
}
