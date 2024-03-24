use std::sync::Arc;

mod client;
mod events;
pub mod util;

use client::Client;
use repositories::{init_repository, RepositoriesError, Repository};
use reqwest::StatusCode;
use toml::{map::Map, Value};
use util::workspace_dir;

#[derive(Clone, Debug)]
pub struct State {
    repository: Repository,
    client: Client,
    config: Config,
}

#[derive(Clone, Debug)]
pub struct Config {
    pause_secs: u64,
    username: String,
}

impl State {
    pub fn new(repository: Repository, client: Client, config: Config) -> Self {
        Self {
            repository,
            client,
            config,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SyncGithubError {
    #[error("Failed to init repository: {}", source)]
    FailedToInitRepository { source: RepositoriesError },

    #[error("Failed to init github client: {}", message)]
    FailedToInitGithubClient { message: String },

    #[error("Failed to call api: {}", source)]
    FailedToCallAPI { source: reqwest::Error },

    #[error("Failed to call api: {}", source)]
    FailedToGetResponse { source: reqwest::Error },

    #[error("Failed status code({}): {}", code, message)]
    FailedStatusCode { code: StatusCode, message: String },

    #[error("Failed to init service: {}", message)]
    FailedToInitService { message: String },
}

pub async fn serve(
    conn_string: &str,
    github_token: &str,
) -> Result<(), SyncGithubError> {
    let config = load_config()?;

    let repository = init_repository(conn_string)
        .await
        .map_err(|e| SyncGithubError::FailedToInitRepository { source: e })?;

    let client = Client::new(github_token.to_string(), &config)?;

    let state_config = init_config(&config)?;

    let state = Arc::new(State::new(repository, client, state_config));

    events::spawn_service_to_get_events(state.clone()).await?;

    Ok(())
}

fn load_config() -> Result<Map<String, Value>, SyncGithubError> {
    let workspace_dir = workspace_dir();
    let config = std::fs::read_to_string(workspace_dir.join("Config.toml"))
        .map_err(|e| SyncGithubError::FailedToInitService {
            message: format!("failed to read Config.toml: {}", e),
        })?;

    let config =
        toml::from_str::<Map<String, Value>>(&config).map_err(|e| {
            SyncGithubError::FailedToInitService {
                message: format!("failed to parse Config.toml: {}", e),
            }
        })?;

    Ok(config)
}

pub fn init_config(
    config: &Map<String, Value>,
) -> Result<Config, SyncGithubError> {
    let github = config.get("github").ok_or_else(|| {
        SyncGithubError::FailedToInitService {
            message: "failed to load github config".to_string(),
        }
    })?;

    let pause_secs = github
        .get("pause_secs")
        .ok_or_else(|| SyncGithubError::FailedToInitService {
            message: "failed to load pause_secs config".to_string(),
        })?
        .as_integer()
        .ok_or_else(|| SyncGithubError::FailedToInitService {
            message: "failed to parse pause_secs config".to_string(),
        })?;

    let username = github
        .get("username")
        .ok_or_else(|| SyncGithubError::FailedToInitService {
            message: "failed to load username config".to_string(),
        })?
        .as_str()
        .ok_or_else(|| SyncGithubError::FailedToInitService {
            message: "failed to parse username config".to_string(),
        })?
        .to_string();

    Ok(Config {
        pause_secs: pause_secs as u64,
        username,
    })
}
