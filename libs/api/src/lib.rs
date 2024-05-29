use std::sync::Arc;

use axum::{middleware, routing::get, routing::post, Router};

use qdrant_client::client::QdrantClient;
use repository::Repository;
use rpc_router::Router as RPCRouter;
use tokio::sync::OnceCell;
use toml::{map::Map, Value};
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
use util::workspace_dir;
use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;
use utoipauto::utoipauto;

use crate::top::{receive, send};

pub mod agent;
mod auth;
pub mod block;
pub mod event;
pub mod healthz;
pub mod not_found;
pub mod nudge;
pub mod page;
pub mod post;
mod request;
mod response;
pub mod runtime;
pub mod search;
pub mod top;
pub mod user;

pub enum ApiError {
    AuthError(String),
    ClientError(String),
    ServerError(String),
}

pub struct ApiState {
    repo: Repository,
    notion: notion_client::endpoints::Client,
    rpc: RPCRouter,
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

static ADMIN_USER: OnceCell<String> = OnceCell::const_new();
static JWKS_URL: OnceCell<String> = OnceCell::const_new();

#[allow(clippy::too_many_arguments)]
pub async fn serve(
    repository: Repository,
    notion_client: notion_client::endpoints::Client,
    rpc: RPCRouter,
    qdrant: QdrantClient,
    cloudflare: cloudflare::models::Models,
    s3: aws_sdk_s3::Client,
    bucket: String,
    config_name: &str,
    admin_user: String,
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

    let config = load_config(config_name)?;

    ADMIN_USER.set(admin_user).unwrap();
    JWKS_URL
        .set(config["auth0"]["jwks_url"].as_str().unwrap().to_string())
        .unwrap();

    let state = Arc::new(ApiState {
        repo: repository.clone(),
        notion: notion_client,
        rpc,
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

    // user
    let user_router = Router::new()
        .route("/", get(user::get_user))
        .route_layer(middleware::from_fn(auth::user_auth))
        .with_state(state.clone());

    // pages
    let page_router = Router::new()
        .route(
            "/:id/generate-cover-image",
            post(page::generate_cover_image),
        )
        .route("/:id/generate-summary", post(page::generate_summarize))
        .route_layer(middleware::from_fn(auth::admin_auth))
        .route("/", get(page::get_pages))
        .route("/:id", get(page::get_page))
        .with_state(state.clone());

    // blocks
    let block_router = Router::new()
        .route("/", get(block::get_blocks))
        .route("/:id", get(block::get_block))
        .with_state(repository.clone());

    // events
    let event_router = Router::new()
        .route("/", get(event::get_events))
        .route("/:id", get(event::get_event))
        .with_state(repository.clone());

    // posts
    let post_router = Router::new()
        .route("/", get(post::get_posts))
        .with_state(repository.clone());

    // top
    let top_router = Router::new()
        .route("/send", get(send))
        .route("/receive", get(receive))
        .with_state(repository.clone());

    // search
    let search_router = Router::new()
        .route("/", get(search::search_text))
        .route("/sse", post(search::search_text_with_sse))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::set_user_id,
        ))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::rate_limit,
        ))
        .route_layer(middleware::from_fn(auth::user_auth))
        .with_state(state.clone());

    // nudge
    let nudge_router = Router::new()
        .route("/", get(search::search_text))
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
        .nest("/user", user_router)
        .nest("/pages", page_router)
        .nest("/blocks", block_router)
        .nest("/events", event_router)
        .nest("/posts", post_router)
        .nest("/search", search_router)
        .nest("/nudge", nudge_router)
        // .nest("/runtime", runtime_router)
        .nest("/top", top_router)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .fallback(not_found::get_404);

    Ok(router)
}

fn load_config(config_name: &str) -> anyhow::Result<Map<String, Value>> {
    let workspace_dir = workspace_dir();
    let config = std::fs::read_to_string(workspace_dir.join(config_name))?;

    let config = toml::from_str::<Map<String, Value>>(&config)?;

    Ok(config)
}
