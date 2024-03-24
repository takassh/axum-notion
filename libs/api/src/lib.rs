use std::fs;

use axum::{http::StatusCode, response::IntoResponse, routing::get, Router};
use repositories::{init_repository, RepositoriesError};
use serde_json::Value;
use tower_http::cors::CorsLayer;
use tracing::{error, info};

use crate::util::workspace_dir;

pub mod block;
pub mod event;
pub mod feed;
pub mod healthz;
pub mod not_found;
pub mod page;
mod util;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{:?}", m)]
    InternalServerError { m: String },
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", self)).into_response()
    }
}

type ApiResponse<T> = Result<T, ApiError>;

pub trait IntoApiResponse<T> {
    fn into_response(self, c: &str) -> ApiResponse<T>;
}

impl<T> IntoApiResponse<T> for Result<T, RepositoriesError> {
    fn into_response(self, c: &str) -> ApiResponse<T> {
        self.map_err(|e| {
            error!("{:?}", e);
            let errors = fs::read_to_string(
                workspace_dir().join("libs/api/src/error.json"),
            )
            .unwrap();
            let parsed: Value = serde_json::from_str(&errors).unwrap();
            let errors = parsed.as_object().unwrap().clone();
            ApiError::InternalServerError {
                m: errors[c].as_str().unwrap().to_string(),
            }
        })
    }
}

pub async fn serve(conn_string: &str) -> ApiResponse<Router> {
    info!("Start API Serving");

    let origins = ["http://localhost:3000".parse().unwrap()];

    let repository = init_repository(conn_string)
        .await
        .into_response("500-001")?;

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

    let router = Router::new()
        .route("/healthz", get(healthz::get_health))
        .nest("/pages", page_router)
        .nest("/blocks", block_router)
        .nest("/events", event_router)
        .layer(CorsLayer::new().allow_origin(origins))
        .fallback(not_found::get_404);

    Ok(router)
}
