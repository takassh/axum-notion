use axum::{routing::get, Router};
use models::{init_repository, ModelsError};

pub mod block;
pub mod healthz;
pub mod not_found;
pub mod page;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Failed to init repository: {}", source)]
    FailedToInitRepository { source: ModelsError },
}

pub async fn serve(conn_string: &str) -> Result<Router, ApiError> {
    let repository = init_repository(conn_string)
        .await
        .map_err(|e| ApiError::FailedToInitRepository { source: e })?;

    // pages
    let page_router = Router::new()
        .route("/", get(page::get_pages))
        .route("/:id", get(page::get_page))
        .fallback(not_found::get_404)
        .with_state(repository.clone());

    // blocks
    let block_router = Router::new()
        .route("/", get(block::get_blocks))
        .route("/:id", get(block::get_block))
        .fallback(not_found::get_404)
        .with_state(repository.clone());

    let router = Router::new()
        .route("/healthz", get(healthz::get_health))
        .nest("/pages", page_router)
        .nest("/blocks", block_router)
        .fallback(not_found::get_404);

    Ok(router)
}
