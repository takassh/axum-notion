use entity::block::Block;
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
}

pub fn serve(repository: Repository) -> Result<Router, RpcError> {
    // Build the Router with the handlers and common resources
    let rpc_router = router_builder!(
        handlers: [find_article_by_word],         // will be turned into routes
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
) -> Result<Option<Block>, RpcError> {
    let page = state.repo.page.find_by_word(&params.word).await?;
    let Some(page) = page else {
        return Ok(None);
    };

    state
        .repo
        .block
        .find_by_notion_page_id(&page.notion_page_id)
        .await
        .map_err(RpcError::RepositoryError)
}