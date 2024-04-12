use std::sync::Arc;

mod client;
mod events;
pub mod util;

use anyhow::Context as _;
use client::Client;
use repository::Repository;
use tokio::task::JoinHandle;
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

pub async fn serve(
    repository: Repository,
    config_name: &str,
    github_token: &str,
) -> anyhow::Result<Vec<JoinHandle<anyhow::Result<()>>>> {
    info!(task = "start github sync");

    let config = load_config(config_name)?;

    let client = Client::new(github_token.to_string(), &config)?;

    let state_config = init_config(&config)?;

    let state = Arc::new(State::new(repository, client, state_config));

    Ok(events::spawn_service_to_get_events(state.clone()))
}

fn load_config(config_name: &str) -> anyhow::Result<Map<String, Value>> {
    let workspace_dir = workspace_dir();
    let config = std::fs::read_to_string(workspace_dir.join(config_name))?;

    let config = toml::from_str::<Map<String, Value>>(&config)?;

    Ok(config)
}

pub fn init_config(config: &Map<String, Value>) -> anyhow::Result<Config> {
    let github = config
        .get("github")
        .context("failed to get github config")?;

    let pause_secs = github
        .get("pause_secs")
        .context("failed to load pause_secs config")?
        .as_integer()
        .context("failed to parse pause_secs config")?;

    let username = github
        .get("username")
        .context("failed to load username config")?
        .as_str()
        .context("failed to parse username config")?
        .to_string();

    Ok(Config {
        pause_secs: pause_secs as u64,
        username,
    })
}
