use anyhow::anyhow;
use chrono::{DateTime, Utc};
use entity::prelude::*;
use notion_client::objects::page::{Page, PageProperty};
use repository::Repository;
use rpc_router::{
    router_builder, Router, RpcHandlerError, RpcParams, RpcResource,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, RpcResource)]
pub struct RpcState {
    repo: Repository,
}

#[derive(Debug, thiserror::Error, RpcHandlerError)]
pub enum RpcError {
    #[error("repository error: {0}")]
    RepositoryError(#[from] anyhow::Error),
    #[error("serde json error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
}

pub fn serve(repository: Repository) -> Result<Router, RpcError> {
    // Build the Router with the handlers and common resources
    let rpc_router = router_builder!(
        handlers: [find_article_by_word,get_current_datetime,get_all_titles_with_created_time],         // will be turned into routes
        resources: [RpcState {repo:repository}] // common resources for all calls
    )
    .build();

    Ok(rpc_router)
}

#[derive(Serialize, Deserialize, RpcParams)]
pub struct ParamsFindByWord {
    word: String,
}
pub async fn find_article_by_word(
    state: RpcState,
    params: ParamsFindByWord,
) -> Result<Option<PageEntity>, RpcError> {
    state
        .repo
        .page
        .find_by_word(&params.word)
        .await
        .map_err(RpcError::RepositoryError)
}

pub async fn get_all_titles_with_created_time(
    state: RpcState,
) -> Result<Vec<(String, String)>, RpcError> {
    let pages = state.repo.page.find_all().await?;
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
