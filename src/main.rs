use std::path::{Path, PathBuf};
use std::process::id;

use anyhow::Context;
use shuttle_runtime::{SecretStore, Secrets};
use tokio::join;
use toml::map::Map;
use toml::Value;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{
    layer::SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

#[shuttle_runtime::main]
async fn main(
    #[Secrets] secret_store: SecretStore,
    #[shuttle_shared_db::Postgres(local_uri = "{secrets.LOCAL_DATABASE_URL}")]
    conn_string: String,
) -> shuttle_axum::ShuttleAxum {
    init_log(secret_store.clone())?;

    let notion_token = secret_store
        .get("NOTION_TOKEN")
        .context("NOTION_TOKEN was not found")?;

    let github_token = secret_store
        .get("GITHUB_TOKEN")
        .context("GITHUB_TOKEN was not found")?;

    let config = secret_store.get("CONFIG").context("CONFIG was not found")?;

    let config_name = &format!("Config{}", config);

    let (notion, github, router) = join!(
        sync_notion::serve(&conn_string, notion_token),
        sync_github::serve(config_name, &conn_string, &github_token),
        api::serve(&conn_string)
    );

    let _ = notion.context("failed to build notion service")?;
    let _ = github.context("failed to build github service")?;
    let router = router.context("failed to build api service")?;

    Ok(router.into())
}

fn init_log(store: SecretStore) -> anyhow::Result<()> {
    let config = load_config()?;

    let grafana = config.get("grafana").context("failed to find grafana")?;

    let use_loki = grafana
        .get("loki")
        .context("failed to find loki")?
        .as_bool()
        .context("failed to parse loki")?;

    if !use_loki {
        tracing_subscriber::fmt().init();

        return Ok(());
    }

    let host = grafana
        .get("host")
        .context("failed to find host")?
        .as_str()
        .context("failed to parse host")?;

    let grafana_user = store.get("GRAFANA_USER").unwrap();
    let grafana_password = store.get("GRAFANA_API_KEY").unwrap();

    use url::Url;

    let url = Url::parse(&format!(
        "https://{grafana_user}:{grafana_password}@{host}"
    ))
    .expect("Failed to parse Grafana URL");

    let (layer, task) = tracing_loki::builder()
        .label("application", "notion-grafana")
        .unwrap()
        .extra_field("pid", format!("{}", id()))
        .unwrap()
        .build_url(url)
        .unwrap();

    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::DEBUG.into())
        .parse("")
        .unwrap();

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::Layer::new())
        .with(layer)
        .init();

    tracing_subscriber::fmt::init();

    tokio::spawn(task);

    Ok(())
}

fn load_config() -> anyhow::Result<Map<String, Value>> {
    let workspace_dir = workspace_dir();
    let config = std::fs::read_to_string(workspace_dir.join("Config.toml"))?;

    let config = toml::from_str::<Map<String, Value>>(&config)?;

    Ok(config)
}

fn workspace_dir() -> PathBuf {
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
