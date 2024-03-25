use std::sync::Arc;

use notion_client::{endpoints::Client, NotionClientError};
use repository::{init_repository, Repository, RepositoryError};
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
    #[error("in std from I/O operations: {}, {}", message, source)]
    StdIoError {
        source: std::io::Error,
        message: String,
    },

    #[error("in toml from deserializing a type: {}, {}", message, source)]
    TomlDeError {
        source: toml::de::Error,
        message: String,
    },

    #[error("in repository: {}, {}", message, source)]
    RepositoryError {
        source: RepositoryError,
        message: String,
    },

    #[error("in notion: {}, {}", message, source)]
    NotionClientError {
        source: Box<NotionClientError>,
        message: String,
    },

    #[error("option: {}", message)]
    Option { message: String },
}

type Response<T> = Result<T, SyncNotionError>;

pub trait IntoResponse<T> {
    fn into_response(self, message: &str) -> Response<T>;
}

impl<T> IntoResponse<T> for Result<T, std::io::Error> {
    fn into_response(self, message: &str) -> Response<T> {
        self.map_err(|e| SyncNotionError::StdIoError {
            source: e,
            message: message.to_string(),
        })
    }
}

impl<T> IntoResponse<T> for Result<T, toml::de::Error> {
    fn into_response(self, message: &str) -> Response<T> {
        self.map_err(|e| SyncNotionError::TomlDeError {
            source: e,
            message: message.to_string(),
        })
    }
}

impl<T> IntoResponse<T> for Result<T, RepositoryError> {
    fn into_response(self, message: &str) -> Response<T> {
        self.map_err(|e| SyncNotionError::RepositoryError {
            source: e,
            message: message.to_string(),
        })
    }
}

impl<T> IntoResponse<T> for Result<T, NotionClientError> {
    fn into_response(self, message: &str) -> Response<T> {
        self.map_err(|e| SyncNotionError::NotionClientError {
            source: Box::new(e),
            message: message.to_string(),
        })
    }
}

impl<T> IntoResponse<T> for Option<T> {
    fn into_response(self, message: &str) -> Response<T> {
        self.ok_or_else(|| SyncNotionError::Option {
            message: message.to_string(),
        })
    }
}

pub async fn serve(
    conn_string: &str,
    notion_token: String,
    notion_db_id: String,
) -> Result<(), SyncNotionError> {
    info!("Start Notion Sync");

    let repository = init_repository(conn_string)
        .await
        .into_response("failed to init repository")?;

    let client = Client::new(notion_token)
        .into_response("failed to init notion client")?;

    let config = load_config()?;

    let pause_secs = config
        .get("notion")
        .into_response("failed to load github config")?
        .get("pause_secs")
        .into_response("failed to load pause_secs config")?
        .as_integer()
        .into_response("failed to parse pause_secs config")?;

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
        .into_response("failed to read Config.toml")?;

    let config = toml::from_str::<Map<String, Value>>(&config)
        .into_response("failed to parse Config.toml")?;

    Ok(config)
}
