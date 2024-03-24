use shuttle_runtime::{Error, SecretStore, Secrets};
use tokio::join;

#[shuttle_runtime::main]
async fn main(
    #[Secrets] secret_store: SecretStore,
    #[shuttle_shared_db::Postgres(local_uri = "{secrets.LOCAL_DATABASE_URL}")]
    conn_string: String,
) -> shuttle_axum::ShuttleAxum {
    if let Some(env) = secret_store.get("ENV") {
        if env == "prod" {
            tracing_subscriber::fmt()
                .with_max_level(tracing::Level::INFO)
                .init();
        }
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
    }

    let Some(notion_token) = secret_store.get("NOTION_TOKEN") else {
        return Err(Error::BuildPanic(
            "NOTION_TOKEN was not found".to_string(),
        ));
    };
    let Some(notion_db_id) = secret_store.get("NOTION_DB_ID") else {
        return Err(Error::BuildPanic(
            "NOTION_DB_ID was not found".to_string(),
        ));
    };
    let Some(github_token) = secret_store.get("GITHUB_TOKEN") else {
        return Err(Error::BuildPanic(
            "GITHUB_TOKEN was not found".to_string(),
        ));
    };

    let (notion, github, router) = join!(
        sync_notion::serve(&conn_string, notion_token, notion_db_id),
        sync_github::serve(&conn_string, &github_token),
        api::serve(&conn_string)
    );

    notion.map_err(|e| Error::BuildPanic(e.to_string()))?;
    github.map_err(|e| Error::BuildPanic(e.to_string()))?;
    let router = router.map_err(|e| Error::BuildPanic(e.to_string()))?;

    Ok(router.into())
}
