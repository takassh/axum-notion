use axum::{routing::get, Router};
use repository::init_repository;
use tower_http::cors::CorsLayer;
use tracing::info;

pub mod block;
pub mod event;
pub mod healthz;
pub mod not_found;
pub mod page;
pub mod post;
mod response;
mod util;

pub enum ApiError {
    ClientError(String),
    ServerError(String),
}

pub async fn serve(conn_string: &str) -> anyhow::Result<Router> {
    info!("Start API Serving");

    let origins = ["http://localhost:3000".parse().unwrap()];

    let repository = init_repository(conn_string).await?;

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

    // events
    let event_router = Router::new()
        .route("/", get(event::get_events))
        .route("/:id", get(event::get_event))
        .fallback(not_found::get_404)
        .with_state(repository.clone());

    // posts
    let post_router = Router::new()
        .route("/", get(post::get_posts))
        .fallback(not_found::get_404)
        .with_state(repository.clone());

    let router = Router::new()
        .route("/healthz", get(healthz::get_health))
        .nest("/pages", page_router)
        .nest("/blocks", block_router)
        .nest("/events", event_router)
        .nest("/posts", post_router)
        .layer(CorsLayer::new().allow_origin(origins))
        .fallback(not_found::get_404);

    Ok(router)
}
