use std::sync::Arc;

use anyhow::Context as _;
use notion_client::endpoints::Client;
use repository::{init_repository, Repository};
use tokio::join;
use toml::{map::Map, Value};
use tracing::info;
use util::workspace_dir;

mod block;
mod page;
mod util;

#[derive(Clone, Debug)]
pub struct State {
    repository: Repository,
    client: Client,
    notion_db_id: String,
    pause_secs: u64,
}

impl State {
    pub fn new(
        repository: Repository,
        client: Client,
        notion_db_id: String,
        pause_secs: u64,
    ) -> Self {
        Self {
            repository,
            client,
            notion_db_id,
            pause_secs,
        }
    }
}

pub async fn serve(
    conn_string: &str,
    notion_token: String,
    notion_db_id: String,
) -> anyhow::Result<()> {
    info!("Start Notion Sync");

    let repository = init_repository(conn_string).await?;

    let client = Client::new(notion_token)?;

    let config = load_config()?;

    let pause_secs = config
        .get("notion")
        .context("failed to load github config")?
        .get("pause_secs")
        .context("failed to load pause_secs config")?
        .as_integer()
        .context("failed to parse pause_secs config")?;

    let state = Arc::new(State::new(
        repository,
        client,
        notion_db_id,
        pause_secs as u64,
    ));

    let (page, block) = join!(
        page::spawn_service_to_get_pages(state.clone()),
        block::spawn_service_to_get_blocks(state.clone())
    );

    page?;
    block?;

    Ok(())
}

fn load_config() -> anyhow::Result<Map<String, Value>> {
    let workspace_dir = workspace_dir();
    let config = std::fs::read_to_string(workspace_dir.join("Config.toml"))?;

    let config = toml::from_str::<Map<String, Value>>(&config)?;

    Ok(config)
}
