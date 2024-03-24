use std::sync::Arc;

use notion_client::{endpoints::Client, NotionClientError};
use repositories::{init_repository, RepositoriesError, Repository};
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

#[derive(Debug, thiserror::Error)]
pub enum SyncNotionError {
    #[error("Failed to init repository: {}", source)]
    FailedToInitRepository { source: RepositoriesError },

    #[error("Failed to init notion client: {}", source)]
    FailedToInitNotionClient { source: NotionClientError },

    #[error("Failed to call repository: {}", source)]
    FailedToCallRepository { source: RepositoriesError },

    #[error("Failed to init service: {}", message)]
    FailedToInitService { message: String },
}

pub async fn serve(
    conn_string: &str,
    notion_token: String,
    notion_db_id: String,
) -> Result<(), SyncNotionError> {
    info!("Start Notion Sync");

    let repository = init_repository(conn_string)
        .await
        .map_err(|e| SyncNotionError::FailedToInitRepository { source: e })?;

    let client = Client::new(notion_token)
        .map_err(|e| SyncNotionError::FailedToInitNotionClient { source: e })?;

    let config = load_config()?;

    let pause_secs = config
        .get("notion")
        .ok_or_else(|| SyncNotionError::FailedToInitService {
            message: "failed to load github config".to_string(),
        })?
        .get("pause_secs")
        .ok_or_else(|| SyncNotionError::FailedToInitService {
            message: "failed to load pause_secs config".to_string(),
        })?
        .as_integer()
        .ok_or_else(|| SyncNotionError::FailedToInitService {
            message: "failed to parse pause_secs config".to_string(),
        })?;

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

fn load_config() -> Result<Map<String, Value>, SyncNotionError> {
    let workspace_dir = workspace_dir();
    let config = std::fs::read_to_string(workspace_dir.join("Config.toml"))
        .map_err(|e| SyncNotionError::FailedToInitService {
            message: format!("failed to read Config.toml: {}", e),
        })?;

    let config =
        toml::from_str::<Map<String, Value>>(&config).map_err(|e| {
            SyncNotionError::FailedToInitService {
                message: format!("failed to parse Config.toml: {}", e),
            }
        })?;

    Ok(config)
}
