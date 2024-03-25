use std::sync::Arc;

mod client;
mod events;
pub mod util;

use client::Client;
use repositories::{init_repository, Repository, RepositoryError};
use reqwest::StatusCode;
use toml::{map::Map, Value};
use tracing::info;
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
    #[error("Failed status code({}): {}", status_code, message)]
    FailedStatusCode {
        status_code: StatusCode,
        message: String,
    },

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

    #[error("in reqwest from processing a Request: {}, {}", message, source)]
    ReqwestError {
        source: reqwest::Error,
        message: String,
    },

    #[error("in repository: {}, {}", message, source)]
    RepositoryError {
        source: RepositoryError,
        message: String,
    },

    #[error("option: {}", message)]
    Option { message: String },
}

type Response<T> = Result<T, SyncGithubError>;

pub trait IntoResponse<T> {
    fn into_response(self, message: &str) -> Response<T>;
}

impl<T> IntoResponse<T> for Result<T, std::io::Error> {
    fn into_response(self, message: &str) -> Response<T> {
        self.map_err(|e| SyncGithubError::StdIoError {
            source: e,
            message: message.to_string(),
        })
    }
}

impl<T> IntoResponse<T> for Result<T, toml::de::Error> {
    fn into_response(self, message: &str) -> Response<T> {
        self.map_err(|e| SyncGithubError::TomlDeError {
            source: e,
            message: message.to_string(),
        })
    }
}

impl<T> IntoResponse<T> for Result<T, reqwest::Error> {
    fn into_response(self, message: &str) -> Response<T> {
        self.map_err(|e| SyncGithubError::ReqwestError {
            source: e,
            message: message.to_string(),
        })
    }
}

impl<T> IntoResponse<T> for Result<T, RepositoryError> {
    fn into_response(self, message: &str) -> Response<T> {
        self.map_err(|e| SyncGithubError::RepositoryError {
            source: e,
            message: message.to_string(),
        })
    }
}

impl<T> IntoResponse<T> for Option<T> {
    fn into_response(self, message: &str) -> Response<T> {
        self.ok_or_else(|| SyncGithubError::Option {
            message: message.to_string(),
        })
    }
}

pub async fn serve(
    conn_string: &str,
    github_token: &str,
) -> Result<(), SyncGithubError> {
    info!("Start Github Sync");

    let config = load_config()?;

    let repository = init_repository(conn_string)
        .await
        .into_response("failed to init repository")?;

    let client = Client::new(github_token.to_string(), &config)?;

    let state_config = init_config(&config)?;

    let state = Arc::new(State::new(repository, client, state_config));

    events::spawn_service_to_get_events(state.clone()).await?;

    Ok(())
}

fn load_config() -> Result<Map<String, Value>, SyncGithubError> {
    let workspace_dir = workspace_dir();
    let config = std::fs::read_to_string(workspace_dir.join("Config.toml"))
        .into_response("failed to read Config.toml")?;

    let config = toml::from_str::<Map<String, Value>>(&config)
        .into_response("failed to parse Config.toml")?;

    Ok(config)
}

pub fn init_config(
    config: &Map<String, Value>,
) -> Result<Config, SyncGithubError> {
    let github = config
        .get("github")
        .into_response("failed to load github config")?;

    let pause_secs = github
        .get("pause_secs")
        .into_response("failed to load pause_secs config")?
        .as_integer()
        .into_response("failed to parse pause_secs config")?;

    let username = github
        .get("username")
        .into_response("failed to load username config")?
        .as_str()
        .into_response("failed to parse username config")?
        .to_string();

    Ok(Config {
        pause_secs: pause_secs as u64,
        username,
    })
}
