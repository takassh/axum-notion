use std::sync::Arc;

use anyhow::Context as _;
use notion_client::endpoints::Client;
use repository::Repository;
use tokio::task::JoinHandle;
use toml::{map::Map, Value};
use tracing::info;
use util::workspace_dir;

mod block;
mod page;
pub mod util;

#[derive(Clone, Debug)]
pub struct State {
    repository: Repository,
    client: Client,
    pause_secs: u64,
}

impl State {
    pub fn new(
        repository: Repository,
        client: Client,
        pause_secs: u64,
    ) -> Self {
        Self {
            repository,
            client,
            pause_secs,
        }
    }
}

pub async fn serve(
    repository: Repository,
    client: notion_client::endpoints::Client,
    config_name: &str,
) -> anyhow::Result<Vec<JoinHandle<anyhow::Result<()>>>> {
    info!(task = "start notion sync");

    let config = load_config(config_name)?;

    let pause_secs = config
        .get("notion")
        .context("failed to load github config")?
        .get("pause_secs")
        .context("failed to load pause_secs config")?
        .as_integer()
        .context("failed to parse pause_secs config")?;

    let state = Arc::new(State::new(repository, client, pause_secs as u64));

    let page_handles = page::spawn_service_to_get_pages(state.clone());
    let block_handles = block::spawn_service_to_get_blocks(state.clone());

    Ok(page_handles.into_iter().chain(block_handles).collect())
}

fn load_config(config_name: &str) -> anyhow::Result<Map<String, Value>> {
    let workspace_dir = workspace_dir();
    let config = std::fs::read_to_string(workspace_dir.join(config_name))?;

    let config = toml::from_str::<Map<String, Value>>(&config)?;

    Ok(config)
}
