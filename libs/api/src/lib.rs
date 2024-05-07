use std::sync::Arc;

use axum::{middleware, routing::get, routing::post, Router};

use qdrant_client::client::QdrantClient;
use repository::Repository;
use tokio::sync::OnceCell;
use toml::{map::Map, Value};
use tower_http::cors::CorsLayer;
use tracing::info;
use util::workspace_dir;
use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;
use utoipauto::utoipauto;

use crate::top::{receive, send};

mod auth;
pub mod block;
pub mod event;
pub mod healthz;
pub mod not_found;
pub mod page;
pub mod post;
mod request;
mod response;
pub mod runtime;
pub mod search;
pub mod top;
pub mod ws;

pub enum ApiError {
    AuthError(String),
    ClientError(String),
    ServerError(String),
}

pub struct ApiState {
    repo: Repository,
    notion: notion_client::endpoints::Client,
    cloudflare: cloudflare::models::Models,
    s3: aws_sdk_s3::Client,
    qdrant: QdrantClient,
    config: Config,
}

pub struct Config {
    pub aws: AWS,
    pub qdrant: Qdrant,
}

pub struct AWS {
    pub bucket: String,
    pub s3_url: String,
}

pub struct Qdrant {
    pub collection: String,
}

static ACCEPT_API_KEY: OnceCell<String> = OnceCell::const_new();

#[allow(clippy::too_many_arguments)]
pub async fn serve(
    repository: Repository,
    notion_client: notion_client::endpoints::Client,
    qdrant: QdrantClient,
    cloudflare: cloudflare::models::Models,
    s3: aws_sdk_s3::Client,
    bucket: String,
    config_name: &str,
    accept_api_key: String,
) -> anyhow::Result<Router> {
    #[utoipauto(paths = "./libs/api/src")]
    #[derive(OpenApi)]
    #[openapi(
        tags(
            (name = "todo", description = "Todo items management API")
        )
    )]
    struct ApiDoc;

    info!(task = "start api serving");

    ACCEPT_API_KEY.set(accept_api_key).unwrap();

    let config = load_config(config_name)?;

    let state = Arc::new(ApiState {
        repo: repository.clone(),
        notion: notion_client,
        cloudflare,
        s3,
        qdrant,
        config: Config {
            aws: AWS {
                bucket,
                s3_url: config["aws"]["s3_url"].as_str().unwrap().to_string(),
            },
            qdrant: Qdrant {
                collection: config["qdrant"]["collection"]
                    .as_str()
                    .unwrap()
                    .to_string(),
            },
        },
    });

    let origins = ["http://localhost:3000".parse().unwrap()];
    // pages
    let page_router = Router::new()
        .route("/", get(page::get_pages))
        .route("/:id", get(page::get_page))
        .route(
            "/:id/generate-cover-image",
            post(page::generate_cover_image),
        )
        .route(
            "/:id/generate-cover-image-from-plain-texts",
            post(page::generate_cover_image_from_plain_texts),
        )
        .route("/:id/generate-summary", post(page::generate_summarize))
        .fallback(not_found::get_404)
        .with_state(state.clone());

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

    // posts
    let ws_router = Router::new()
        .route("/", get(ws::ws))
        .fallback(not_found::get_404)
        .with_state(state.clone());

    // search
    let search_router = Router::new()
        .route("/", get(search::search_text))
        .fallback(not_found::get_404)
        .with_state(state.clone());

    // runtime
    // let _ = Router::new().route("/", post(runtime::post_code));

    let router = Router::new()
        .merge(
            SwaggerUi::new("/swagger-ui")
                .url("/api-docs/openapi.json", ApiDoc::openapi()),
        )
        .merge(Redoc::with_url("/redoc", ApiDoc::openapi()))
        .merge(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
        .route("/healthz", get(healthz::get_health))
        .nest("/pages", page_router)
        .nest("/blocks", block_router)
        .nest("/events", event_router)
        .nest("/posts", post_router)
        .nest("/search", search_router)
        // .nest("/runtime", runtime_router)
        .route_layer(middleware::from_fn(auth::auth))
        .nest("/ws", ws_router)
        .nest("/top", top_router)
        .layer(CorsLayer::new().allow_origin(origins))
        .fallback(not_found::get_404);

    Ok(router)
}

fn load_config(config_name: &str) -> anyhow::Result<Map<String, Value>> {
    let workspace_dir = workspace_dir();
    let config = std::fs::read_to_string(workspace_dir.join(config_name))?;

    let config = toml::from_str::<Map<String, Value>>(&config)?;

    Ok(config)
}
