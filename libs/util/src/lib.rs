use std::path::{Path, PathBuf};

use anyhow::Context;
use toml::{map::Map, Value};

pub fn workspace_dir() -> PathBuf {
    let output = std::process::Command::new(env!("CARGO"))
        .arg("locate-project")
        .arg("--workspace")
        .arg("--message-format=plain")
        .output()
        .unwrap()
        .stdout;
    let cargo_path = Path::new(std::str::from_utf8(&output).unwrap().trim());
    cargo_path.parent().unwrap().to_path_buf()
}

pub fn load_config(config_name: &str) -> anyhow::Result<Map<String, Value>> {
    let workspace_dir = workspace_dir();
    let config = std::fs::read_to_string(workspace_dir.join(config_name))?;

    let config = toml::from_str::<Map<String, Value>>(&config)?;

    Ok(config)
}

pub fn load_env() -> anyhow::Result<Map<String, Value>> {
    let workspace_dir = workspace_dir();
    let secrets = std::fs::read_to_string(workspace_dir.join("Secrets.toml"))
        .context("failed to read Secrets.toml")?;

    toml::from_str::<Map<String, Value>>(&secrets)
        .context("failed to parse Secrets.toml")
}
