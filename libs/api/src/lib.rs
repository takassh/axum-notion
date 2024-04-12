use axum::{routing::get, Router};

use repository::Repository;
use tower_http::cors::CorsLayer;
use tracing::info;
use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;
use utoipauto::utoipauto;

use crate::top::{receive, send};

pub mod block;
pub mod event;
pub mod healthz;
pub mod not_found;
pub mod page;
pub mod post;
mod response;
pub mod top;
mod util;

pub enum ApiError {
    ClientError(String),
    ServerError(String),
}

pub async fn serve(repository: Repository) -> anyhow::Result<Router> {
    #[utoipauto(paths = "./libs/api/src")]
    #[derive(OpenApi)]
    #[openapi(
        tags(
            (name = "todo", description = "Todo items management API")
        )
    )]
    struct ApiDoc;

    info!(task = "start api serving");

    let origins = ["http://localhost:3000".parse().unwrap()];

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

    // top
    let top_router = Router::new()
        .route("/send", get(send))
        .route("/receive", get(receive))
        .with_state(repository.clone());

    let router = Router::new()
        .merge(
            SwaggerUi::new("/swagger-ui")
                .url("/api-docs/openapi.json", ApiDoc::openapi()),
        )
        .merge(Redoc::with_url("/redoc", ApiDoc::openapi()))
        .merge(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
        .route("/healthz", get(healthz::get_health))
        .nest("/top", top_router)
        .nest("/pages", page_router)
        .nest("/blocks", block_router)
        .nest("/events", event_router)
        .nest("/posts", post_router)
        .layer(CorsLayer::new().allow_origin(origins))
        .fallback(not_found::get_404);

    Ok(router)
}
