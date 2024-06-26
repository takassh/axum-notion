use std::net::{Ipv4Addr, SocketAddr};

use anyhow::Context;
use api::serve;
use aws_sdk_s3::config::Credentials;
use langfuse::apis::configuration;
use repository::Repository;
use tokio::net::TcpListener;
use toml::{map::Map, Value};
use util::{load_config, workspace_dir};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    let secrets = load_env()?;
    let conn_string =
        secrets.get("LOCAL_DATABASE_URL").unwrap().as_str().unwrap();

    let notion_token = secrets.get("NOTION_TOKEN").unwrap().as_str().unwrap();
    let notion_client =
        notion_client::endpoints::Client::new(notion_token.to_string())
            .context("failed to build notion client")?;

    let access_key_id =
        secrets.get("AWS_ACCESS_KEY_ID").unwrap().as_str().unwrap();
    let secret_access_key = secrets
        .get("AWS_SECRET_ACCESS_KEY")
        .unwrap()
        .as_str()
        .unwrap();
    let aws_url = secrets.get("AWS_URL").unwrap().as_str().unwrap();
    let bucket = secrets.get("BUCKET").unwrap().as_str().unwrap();
    let credentials =
        Credentials::new(access_key_id, secret_access_key, None, None, "");
    let cfg = aws_config::from_env()
        .endpoint_url(aws_url)
        .region("auto")
        .credentials_provider(credentials)
        .load()
        .await;
    let s3 = aws_sdk_s3::Client::new(&cfg);

    let admin_user = secrets.get("ADMIN_USER").unwrap().as_str().unwrap();

    let config_name = &format!(
        "Config{}",
        secrets
            .get("CONFIG")
            .context("CONFIG was not found")?
            .as_str()
            .unwrap()
    );

    let cloudflare = cloudflare::models::Models::new(
        secrets
            .get("CLOUDFLARE_ACCOUNT_ID")
            .unwrap()
            .as_str()
            .unwrap(),
        secrets.get("CLOUDFLARE_TOKEN").unwrap().as_str().unwrap(),
    );

    let config = load_config(config_name)?;

    let repository = Repository::new(conn_string).await?.with_session(
        redis::Client::open(format!(
            "rediss://{}:{}@{}:{}",
            config
                .get("upstash")
                .unwrap()
                .get("username")
                .unwrap()
                .as_str()
                .unwrap(),
            secrets.get("REDIS_PASSWORD").unwrap().as_str().unwrap(),
            config
                .get("upstash")
                .unwrap()
                .get("endpoint")
                .unwrap()
                .as_str()
                .unwrap(),
            config
                .get("upstash")
                .unwrap()
                .get("port")
                .unwrap()
                .as_integer()
                .unwrap(),
        ))
        .unwrap(),
    );

    let rpc = rpc::serve(
        config_name,
        repository.clone(),
        qdrant_client::client::QdrantClient::from_url(
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
        .unwrap(),
        cloudflare.clone(),
    )?;
    let router = serve(
        secrets.get("ENV").unwrap().as_str().unwrap().to_string(),
        repository,
        notion_client,
        rpc,
        cloudflare,
        s3,
        qdrant_client::client::QdrantClient::from_url(
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
        .unwrap(),
        configuration::Configuration {
            base_path: config
                .get("langfuse")
                .unwrap()
                .get("base_url")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string(),
            basic_auth: Some((
                secrets
                    .get("LANGFUSE_PUBLIC_KEY")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_string(),
                Some(
                    secrets
                        .get("LANGFUSE_SECRET_KEY")
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .to_string(),
                ),
            )),
            ..Default::default()
        },
        bucket.to_string(),
        config_name,
        admin_user.to_string(),
    )
    .await?;

    let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 8000));
    let listener = TcpListener::bind(&address).await?;
    Ok(axum::serve(
        listener,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?)
}

fn load_env() -> anyhow::Result<Map<String, Value>> {
    let workspace_dir = workspace_dir();
    let secrets =
        std::fs::read_to_string(workspace_dir.join("Secrets.dev.toml"))
            .context("failed to read Secrets.dev.toml")?;

    toml::from_str::<Map<String, Value>>(&secrets)
        .context("failed to parse Secrets.dev.toml")
}
