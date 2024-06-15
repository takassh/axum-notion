use std::sync::Arc;

use anyhow::anyhow;
use chrono::{DateTime, Utc};
use cloudflare::models::text_embeddings::{
    StringOrArray, TextEmbeddings, TextEmbeddingsRequest,
};
use entity::prelude::*;
use notion_client::objects::page::{Page, PageProperty};
use qdrant_client::{
    client::QdrantClient,
    qdrant::{
        condition::ConditionOneOf, r#match::MatchValue,
        with_payload_selector::SelectorOptions, Condition, FieldCondition,
        Filter, Match, PayloadIncludeSelector, ScoredPoint, SearchPoints,
        WithPayloadSelector,
    },
};
use repository::Repository;
use rpc_router::{
    router_builder, Router, RpcHandlerError, RpcParams, RpcResource,
};
use serde::{Deserialize, Serialize};
use util::load_config;

#[derive(Clone, RpcResource)]
pub struct RpcState {
    config: Config,
    repo: Repository,
    qdrant: Arc<QdrantClient>,
    cloudflare: cloudflare::models::Models,
}

#[derive(Clone)]
pub struct Config {
    pub qdrant: Qdrant,
}

#[derive(Clone)]
pub struct Qdrant {
    pub collection: String,
}

#[derive(Debug, thiserror::Error, RpcHandlerError)]
pub enum RpcError {
    #[error("repository error: {0}")]
    RepositoryError(#[from] anyhow::Error),
    #[error("serde json error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
}

pub fn serve(
    config_name: &str,
    repository: Repository,
    qdrant: QdrantClient,
    cloudflare: cloudflare::models::Models,
) -> Result<Router, RpcError> {
    let config = load_config(config_name)?;

    // Build the Router with the handlers and common resources
    let rpc_router = router_builder!(
        handlers: [get_article_summary,get_article_detail,get_current_datetime,get_article_titles_with_date],         // will be turned into routes
        resources: [RpcState {config:Config{qdrant:Qdrant {
            collection: config["qdrant"]["collection"]
                .as_str()
                .unwrap()
                .to_string(),
        }} ,repo:repository,qdrant:Arc::new(qdrant),cloudflare}] // common resources for all calls
    )
    .build();

    Ok(rpc_router)
}

#[derive(Serialize, Deserialize, RpcParams)]
pub struct ParamsFindByWord {
    query: String,
}
pub async fn get_article_summary(
    state: RpcState,
    params: ParamsFindByWord,
) -> Result<Option<PageEntity>, RpcError> {
    let results = retrieve_from_vector_db(&state, params.query).await?;

    let mut page: Option<PageEntity> = None;
    for result in results.iter() {
        if result.score < 0.6 {
            continue;
        }

        // Take page id
        let Some(page_id) = result.payload.get("page_id") else {
            continue;
        };
        let Some(page_id) = page_id.as_str() else {
            continue;
        };

        page = state
            .repo
            .page
            .find_by_id(page_id)
            .await
            .map_err(RpcError::RepositoryError)?;

        if page.is_some() {
            break;
        }
    }

    Ok(page)
}

#[derive(Serialize, Deserialize, RpcParams)]
pub struct ParamsGetArticleFullTextsByTitle {
    query: String,
}
pub async fn get_article_detail(
    state: RpcState,
    params: ParamsGetArticleFullTextsByTitle,
) -> Result<Option<BlockEntity>, RpcError> {
    let results = retrieve_from_vector_db(&state, params.query).await?;

    let mut block = None;
    for result in results.iter() {
        if result.score < 0.7 {
            continue;
        }

        // Take page id
        let Some(page_id) = result.payload.get("page_id") else {
            continue;
        };
        let Some(page_id) = page_id.as_str() else {
            continue;
        };

        block = state
            .repo
            .block
            .find_by_notion_page_id(page_id)
            .await
            .map_err(RpcError::RepositoryError)?;

        if block.is_some() {
            break;
        }
    }

    Ok(block)
}

async fn retrieve_from_vector_db(
    state: &RpcState,
    text: String,
) -> Result<Vec<ScoredPoint>, RpcError> {
    let embedding = state
        .cloudflare
        .bge_small_en_v1_5(TextEmbeddingsRequest {
            text: StringOrArray::String(text),
        })
        .await?;

    let Some(vector) = embedding.result.data.first() else {
        return Ok(vec![]);
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
                        fields: vec!["page_id".to_string()],
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

    Ok(search_result.result)
}

pub async fn get_article_titles_with_date(
    state: RpcState,
) -> Result<Vec<(String, String)>, RpcError> {
    let pages = state.repo.page.find_paginate(0, 10, None, None).await?;
    Ok(pages
        .iter()
        .flat_map(|page| {
            let page = serde_json::from_str::<Page>(&page.contents)
                .map_err(RpcError::SerdeJsonError)?;
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
                    return Err(RpcError::RepositoryError(anyhow!(
                        "title not found in page properties"
                    )));
                };

            Ok(title_and_date)
        })
        .collect())
}

pub async fn get_current_datetime() -> Result<DateTime<Utc>, RpcError> {
    Ok(chrono::Utc::now())
}
