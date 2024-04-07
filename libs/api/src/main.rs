use std::net::{Ipv4Addr, SocketAddr};

use anyhow::Context;
use api::serve;
use tokio::net::TcpListener;
use toml::{map::Map, Value};
use util::workspace_dir;

mod util;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let secrets = load_env()?;
    let conn_string =
        secrets.get("LOCAL_DATABASE_URL").unwrap().as_str().unwrap();

    let router = serve(conn_string).await?;

    let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 8000));
    let listener = TcpListener::bind(&address).await?;
    Ok(axum::serve(listener, router.into_make_service()).await?)
}

fn load_env() -> anyhow::Result<Map<String, Value>> {
    let workspace_dir = workspace_dir();
    let secrets = std::fs::read_to_string(workspace_dir.join("Secrets.toml"))
        .context("failed to read Secrets.toml")?;

    toml::from_str::<Map<String, Value>>(&secrets)
        .context("failed to parse Secrets.toml")
}