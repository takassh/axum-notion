use shuttle_runtime::Error;
use shuttle_secrets::SecretStore;
use tracing::info;
use tracing_subscriber;

#[shuttle_runtime::main]
async fn main(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
    #[shuttle_shared_db::Postgres(local_uri = "{secrets.LOCAL_DATABASE_URL}")] conn_string: String,
) -> shuttle_axum::ShuttleAxum {
    if let Some(is_on_shuttle) = secret_store.get("SHUTTLE") {
        if is_on_shuttle == "true" {
            tracing_subscriber::fmt()
                .with_max_level(tracing::Level::INFO)
                .init();
        }
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    }

    info!("Server listening");

    let Some(notion_token) = secret_store.get("NOTION_TOKEN") else {
        return Err(Error::BuildPanic("NOTION_TOKEN was not found".to_string()));
    };
    let Some(notion_db_id) = secret_store.get("NOTION_DB_ID") else {
        return Err(Error::BuildPanic("NOTION_DB_ID was not found".to_string()));
    };
    let Some(pause_secs) = secret_store.get("PAUSE_SECS") else {
        return Err(Error::BuildPanic("PAUSE_SECS was not found".to_string()));
    };
    let Ok(pause_secs) = pause_secs.parse() else {
        return Err(Error::BuildPanic("PAUSE_SECS was not u64".to_string()));
    };

    sync_notion::serve(&conn_string, notion_token, notion_db_id, pause_secs)
        .await
        .map_err(|e| Error::BuildPanic(e.to_string()))?;

    let router = api::serve(&conn_string)
        .await
        .map_err(|e| Error::BuildPanic(e.to_string()))?;

    Ok(router.into())
}
