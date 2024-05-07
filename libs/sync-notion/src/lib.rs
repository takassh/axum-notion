use std::sync::Arc;

use anyhow::Context as _;
use notion_client::endpoints::Client;
use qdrant_client::qdrant::{
    vectors_config::Config, CreateCollection, VectorParams, VectorsConfig,
};
use repository::Repository;
use tokio::task::JoinHandle;
use tracing::info;

mod block;
mod page;

pub struct State {
    repository: Repository,
    client: Client,
    cloudflare: cloudflare::models::Models,
    qdrant: qdrant_client::client::QdrantClient,
    pause_secs: u64,
    collention: String,
}

impl State {
    pub fn new(
        repository: Repository,
        client: Client,
        cloudflare: cloudflare::models::Models,
        qdrant: qdrant_client::client::QdrantClient,
        pause_secs: u64,
        collention: String,
    ) -> Self {
        Self {
            repository,
            client,
            cloudflare,
            qdrant,
            pause_secs,
            collention,
        }
    }
}

pub async fn serve(
    repository: Repository,
    client: notion_client::endpoints::Client,
    cloudflare: cloudflare::models::Models,
    qdrant: qdrant_client::client::QdrantClient,
    config_name: &str,
) -> anyhow::Result<Vec<JoinHandle<anyhow::Result<()>>>> {
    info!(task = "start notion sync");

    let config = util::load_config(config_name)?;

    let pause_secs = config
        .get("notion")
        .context("failed to load github config")?
        .get("pause_secs")
        .context("failed to load pause_secs config")?
        .as_integer()
        .context("failed to parse pause_secs config")?;

    let collection_name = config
        .get("qdrant")
        .unwrap()
        .get("collection")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();

    if !qdrant
        .collection_exists(collection_name.clone())
        .await
        .unwrap()
    {
        qdrant
            .create_collection(&CreateCollection {
                collection_name: collection_name.clone(),
                vectors_config: Some(VectorsConfig {
                    config: Some(Config::Params(VectorParams {
                        size: 384,
                        distance: 1,
                        ..Default::default()
                    })),
                }),
                ..Default::default()
            })
            .await
            .unwrap();
    }

    let state = Arc::new(State::new(
        repository,
        client,
        cloudflare,
        qdrant,
        pause_secs as u64,
        collection_name,
    ));

    let page_handles = page::spawn_service_to_get_pages(state.clone());
    let block_handles = block::spawn_service_to_get_blocks(state.clone());

    Ok(page_handles.into_iter().chain(block_handles).collect())
}
