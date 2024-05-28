use std::process::id;

use anyhow::Context;
use aws_sdk_s3::config::Credentials;
use repository::Repository;
use shuttle_persist::PersistInstance;
use shuttle_runtime::{SecretStore, Secrets};
use tokio::join;

use tracing::level_filters::LevelFilter;
use tracing_subscriber::{
    layer::SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

#[shuttle_runtime::main]
async fn main(
    #[Secrets] secret_store: SecretStore,
    #[shuttle_shared_db::Postgres(local_uri = "{secrets.LOCAL_DATABASE_URL}")]
    conn_string: String,
    #[shuttle_persist::Persist] persist: PersistInstance,
) -> shuttle_axum::ShuttleAxum {
    let notion_token = secret_store.get("NOTION_TOKEN").unwrap();

    let github_token = secret_store.get("GITHUB_TOKEN").unwrap();

    let cloudflare_token = secret_store.get("CLOUDFLARE_TOKEN").unwrap();
    let cloudflare_account_id =
        secret_store.get("CLOUDFLARE_ACCOUNT_ID").unwrap();

    let access_key_id = secret_store.get("AWS_ACCESS_KEY_ID").unwrap();
    let secret_access_key = secret_store.get("AWS_SECRET_ACCESS_KEY").unwrap();
    let aws_url = secret_store.get("AWS_URL").unwrap();
    let bucket = secret_store.get("BUCKET").unwrap();

    let admin_user = secret_store.get("ADMIN_USER").unwrap();

    let config_name =
        &format!("Config{}", secret_store.get("CONFIG").unwrap().as_str());
    let config = util::load_config(config_name)?;

    let repository = Repository::new(&conn_string)
        .await?
        .with_session(
            redis::Client::open(format!(
                "rediss://{}:{}@{}:{}",
                config
                    .get("upstash")
                    .unwrap()
                    .get("username")
                    .unwrap()
                    .as_str()
                    .unwrap(),
                secret_store.get("REDIS_PASSWORD").unwrap(),
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
        )
        .with_cache(persist);

    let notion_client =
        notion_client::endpoints::Client::new(notion_token.clone())
            .context("failed to build notion client")?;

    let cloudflare = cloudflare::models::Models::new(
        &cloudflare_account_id,
        &cloudflare_token,
    );

    let qdrant1 = qdrant_client::client::QdrantClient::from_url(
        config
            .get("qdrant")
            .unwrap()
            .get("base_url")
            .unwrap()
            .as_str()
            .unwrap(),
    )
    .with_api_key(secret_store.get("QDRANT_API_KEY").unwrap())
    .build()
    .unwrap();
    let qdrant2 = qdrant_client::client::QdrantClient::from_url(
        config
            .get("qdrant")
            .unwrap()
            .get("base_url")
            .unwrap()
            .as_str()
            .unwrap(),
    )
    .with_api_key(secret_store.get("QDRANT_API_KEY").unwrap())
    .build()
    .unwrap();

    let credentials =
        Credentials::new(access_key_id, secret_access_key, None, None, "");
    let cfg = aws_config::from_env()
        .endpoint_url(aws_url)
        .region("auto")
        .credentials_provider(credentials)
        .load()
        .await;
    let s3 = aws_sdk_s3::Client::new(&cfg);

    init_log(secret_store.clone(), config_name)?;

    let (notion, github, router) = join!(
        sync_notion::serve(
            repository.clone(),
            notion_client.clone(),
            cloudflare.clone(),
            qdrant1,
            config_name
        ),
        sync_github::serve(repository.clone(), config_name, &github_token),
        api::serve(
            repository.clone(),
            notion_client,
            rpc::serve(repository).unwrap(),
            qdrant2,
            cloudflare,
            s3,
            bucket,
            config_name,
            admin_user,
        )
    );

    let _ = notion.context("failed to build notion service")?;
    let _ = github.context("failed to build github service")?;
    let router = router.context("failed to build api service")?;

    Ok(router.into())
}

fn init_log(store: SecretStore, config_name: &str) -> anyhow::Result<()> {
    let config = util::load_config(config_name)?;

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
        .with_default_directive(LevelFilter::INFO.into())
        .parse("")
        .unwrap();

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::Layer::new())
        .with(layer)
        .init();

    tokio::spawn(task);

    Ok(())
}
