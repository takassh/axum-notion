use anyhow::Context as _;
use futures::future::join_all;
use repository::Repository;
use std::fs::OpenOptions;
use sync_notion::{serve, util::workspace_dir};
use toml::{map::Map, Value};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let out_file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("log.txt")
        .unwrap();

    tracing_subscriber::fmt().with_writer(out_file).init();

    let secrets = load_env()?;

    let conn_string =
        secrets.get("LOCAL_DATABASE_URL").unwrap().as_str().unwrap();
    let notion_token = secrets.get("NOTION_TOKEN").unwrap().as_str().unwrap();

    let repository = Repository::new(conn_string).await?;

    let handles = serve(repository, notion_token.to_string()).await?;

    let _ = join_all(handles).await;

    return Ok(());
}

fn load_env() -> anyhow::Result<Map<String, Value>> {
    let workspace_dir = workspace_dir();
    let secrets = std::fs::read_to_string(workspace_dir.join("Secrets.toml"))
        .context("failed to read Secrets.toml")?;

    toml::from_str::<Map<String, Value>>(&secrets)
        .context("failed to parse Secrets.toml")
}
