use std::sync::Arc;

use notion_client::{endpoints::Client, NotionClientError};
use repositories::{init_repository, RepositoriesError, Repository};
use toml::{map::Map, Value};
mod block;
mod page;

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
    config: &Map<String, Value>,
) -> Result<(), SyncNotionError> {
    let repository = init_repository(conn_string)
        .await
        .map_err(|e| SyncNotionError::FailedToInitRepository { source: e })?;

    let client = Client::new(notion_token)
        .map_err(|e| SyncNotionError::FailedToInitNotionClient { source: e })?;

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

    page::spawn_service_to_get_pages(state.clone()).await?;
    block::spawn_service_to_get_blocks(state.clone()).await
}
