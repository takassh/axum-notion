use crate::agent::{question_and_answer::QuestionAnswerAgent, Agent};
use anyhow::anyhow;
use async_stream::stream;
use axum::{
    extract::State,
    response::{sse::Event, Sse},
    Extension, Json,
};
use cloudflare::models::text_embeddings::{
    StringOrArray, TextEmbeddings, TextEmbeddingsRequest,
};
use entity::prelude::*;
use futures_util::Stream;
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
    ApiState,
};

use self::request::SearchParam;

pub mod request;

pub async fn search_text_with_sse(
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<ApiState>>,
    Json(params): Json<SearchParam>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let titles_with_date = get_recent_titles_with_date(&state).await;
    let stream = stream! {
        let Ok(titles_with_date) = titles_with_date else{
            error!(
                task = "get recent titles with date",
                error = titles_with_date.unwrap_err().to_string(),
            );
            return;
        };

        let titles_with_date = titles_with_date.iter().map(|(title,date)|format!("- {}, created at {}",title,date)).collect::<Vec<_>>().join("\n");
        let titles_with_date_context =format!("## Article titles sorted by date\n{}",titles_with_date);

        let keyword_generator = QuestionAnswerAgent::new(
            state.cloudflare.clone(),
            format!(r#"# Instructions
            You will answer keywords to search about user and other assistant's conversation.
            Your answer is the keywords with comma separated.
            Never forget your answer must be the keywords. Only respond the keywords.
            Answer variety words.
            You're placed on blog site.
            # Recent articles
            {}
            "#,titles_with_date_context),
            params.history.clone(),
            Some(20),
            );

            let keyword = keyword_generator.prompt(&format!("{}\nOnly answer the keywords",&params.prompt),None).await;
            let Ok(keyword) = keyword else {
                error!(
                    task = "keyword",
                    error = keyword.unwrap_err().to_string(),
                );
                return;
            };
            let keyword = &keyword[0];

        let function_call_agent = FunctionCallAgent::new(
            state.cloudflare.clone(),
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
        ],
        params.history.clone(),
        Some(0.),
        Some(0.),
        Some(1.),
    );

    let mut vector_result = vec![];
    let mut all_page_ids = vec![];
    let keywords = keyword.response.split(',').take(3);
    for keyword in keywords{
        if keyword.is_empty(){
            continue;
        }
        let result = retriever(&state, keyword).await;
        let Ok((result,mut page_ids)) = result else {
            error!(
                task = "get context by retriever",
                error = result.unwrap_err().to_string(),
            );
            continue;
        };


        if !result.is_empty(){
            vector_result.push(format!("## Vector search result with {}\n{}",keyword,result.iter().map(|c|format!("1. {}",c)).collect::<Vec<_>>().join("\n")));
            all_page_ids.append(&mut page_ids);
        } else {
            vector_result.push(format!("## Vector search result with {}\nNot found",keyword));
        }
    }


    let context = format!("## Possible search keywords\n{}",keyword.response);

        let tool_calls = function_call_agent.prompt(&format!("{}\nYour answer starts with <tool_call>",&params.prompt),Some(&context)).await;
        let Ok(tool_calls) = tool_calls else {
            error!(
                task = "function call prompt",
                error = tool_calls.unwrap_err().to_string(),
            );
            return;
        };

        let tool_calls = tool_calls.into_iter().take(3);


        let mut function_result = vec![];
        let mut page_ids = vec![];
        for tool_call in tool_calls.clone(){
            let params = json!(&tool_call.arguments);
            let response = state.rpc.call_route(None,tool_call.clone().name,Some(params.clone())).await;
            let Ok(CallResponse { id: _, method, value }) = response else{
                error!(
                    task = "call route",
                    error = response.unwrap_err().to_string(),
                );
                continue;
            };

            match method.as_str(){
                "get_article_summary" => {
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
                        function_result.push(format!("## Article summary search results with {}\nNot found",params.get("query").unwrap()));
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

                        function_result.push(format!("## Article summary search results with {}\n### title\n{}\n### summary\n{}",params.get("query").unwrap(),title,summary));
                            page_ids.push(notion_page_id);
                }
                "get_current_datetime" => {
                    function_result.push(format!("## Current datetime\n{}",value));
                }
                "get_article_detail" => {
                    let block = serde_json::from_value::<Option<entity::block::Block>>(value.clone());
                    let Ok(block) = block else{
                        error!(
                            task = "parse block entity",
                            value = value.to_string(),
                            error = block.unwrap_err().to_string(),
                        );
                        continue;
                    };
                    let Some(entity::block::Block{notion_page_id,updated_at:_,contents, }) = block else{
                        function_result.push(format!("## Article detail search results with {}\nNot found",params.get("query").unwrap()));
                        continue;
                    };

                    let blocks = serde_json::from_str::<Vec<Block>>(&contents);
                    let Ok(blocks) = blocks else{
                        error!(
                            task = "parse blocks",
                            contents = contents,
                            error = blocks.unwrap_err().to_string(),
                        );
                        continue;
                    };

                    let plain_text = blocks
                        .into_iter()
                        .flat_map(|b| b.block_type.plain_text()).flatten()
                        .collect::<Vec<_>>().join("");

                    function_result.push(format!("## Article detail search results with {}\n### full texts\n{}",params.get("query").unwrap(),plain_text));
                    page_ids.push(notion_page_id);
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
            format!(r#"# Instructions
            You will answer user's prompt. Never lie. Ask for more information if you need.
            You can use given resources if needed.
            When you use knowledge other than given context, you should say it explicitly.
            Takashi made you. He is a software engineer and the owner of the site. You are placed on his blog site.
            Your name is takashi AI. Be concise and informative.
            # Recent articles
            {}"#,titles_with_date_context),
            params.history.clone(),
            None,
            );


        let _prompt = params.prompt.clone();
        let context = format!("{}\n{}",vector_result.join("\n"),function_result.join("\n"));
        let _context = context.clone();
        tokio::spawn(async move{
            let mut question_answer = question_answer_agent.prompt_with_stream(&_prompt,Some(&_context)).await;
            loop{
                let data = question_answer.next().await.transpose();
                let Ok(data) = data else {
                    error!(
                        task = "question answer",
                        error = data.unwrap_err().to_string(),
                    );
                    break;
                };
                if let Some(data) = data{
                    for d in data{
                        let result = message_tx.send(d.response).await;
                        if let Err(err) = result {
                            error!(
                                task = "send message event",
                                error = err.to_string()
                            );
                        }
                    }
                } else{
                    // Finish asnwer
                    break;
                }
            }
    });

        // Take pages from page ids
        let _state = state.clone();
        tokio::spawn(async move{
            let mut pages = vec![];
            for id in &all_page_ids{
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

async fn get_recent_titles_with_date(
    state: &Arc<ApiState>,
) -> anyhow::Result<Vec<(String, String)>> {
    let pages = state.repo.page.find_paginate(0, 10, None, None).await?;
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
                    let date = page.created_time.to_rfc3339();

                    (title, date)
                } else {
                    return Err(anyhow!("title not found in page properties"));
                };

            Ok(title_and_date)
        })
        .collect())
}
