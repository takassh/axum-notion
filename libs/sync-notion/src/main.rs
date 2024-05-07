use anyhow::Context as _;
use futures::future::join_all;
use repository::Repository;
use std::fs::OpenOptions;
use sync_notion::serve;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let out_file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("log.txt")
        .unwrap();

    tracing_subscriber::fmt().with_writer(out_file).init();

    let secrets = util::load_env()?;

    let conn_string =
        secrets.get("LOCAL_DATABASE_URL").unwrap().as_str().unwrap();
    let notion_token = secrets.get("NOTION_TOKEN").unwrap().as_str().unwrap();

    let config_name = &format!(
        "Config{}",
        secrets
            .get("CONFIG")
            .context("CONFIG was not found")?
            .as_str()
            .unwrap()
    );

    let repository = Repository::new(conn_string).await?;

    let notion_client =
        notion_client::endpoints::Client::new(notion_token.to_string())
            .unwrap();

    let cloudflare = cloudflare::models::Models::new(
        secrets
            .get("CLOUDFLARE_ACCOUNT_ID")
            .unwrap()
            .as_str()
            .unwrap(),
        secrets.get("CLOUDFLARE_TOKEN").unwrap().as_str().unwrap(),
    );

    let config = util::load_config(config_name)?;

    let qdrant = qdrant_client::client::QdrantClient::from_url(
        config
            .get("qdrant")
            .unwrap()
            .get("base_url")
            .unwrap()
            .as_str()
            .unwrap(),
    )
    .with_api_key(secrets.get("QDRANT_API_KEY").unwrap().as_str().unwrap())
    .build()
    .unwrap();

    let handles =
        serve(repository, notion_client, cloudflare, qdrant, config_name)
            .await?;

    let _ = join_all(handles).await;

    return Ok(());
}
