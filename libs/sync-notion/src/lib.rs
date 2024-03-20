use std::sync::Arc;

use notion_client::{endpoints::Client, NotionClientError};
use repositories::{init_repository, RepositoriesError, Repository};
mod block;
mod page;

#[derive(Clone, Debug)]
pub struct Config {
    repository: Repository,
    client: Client,
    notion_db_id: String,
    pause_secs: u64,
}

impl Config {
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
}

pub async fn serve(
    conn_string: &str,
    notion_token: String,
    notion_db_id: String,
    pause_secs: u64,
) -> Result<(), SyncNotionError> {
    let repository = init_repository(conn_string)
        .await
        .map_err(|e| SyncNotionError::FailedToInitRepository { source: e })?;

    let client = Client::new(notion_token)
        .map_err(|e| SyncNotionError::FailedToInitNotionClient { source: e })?;

    let state = Arc::new(Config::new(repository, client, notion_db_id, pause_secs));

    page::spawn_service_to_get_pages(state.clone()).await?;
    block::spawn_service_to_get_blocks(state.clone()).await
}
