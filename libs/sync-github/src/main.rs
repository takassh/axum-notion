use std::fs::OpenOptions;
use sync_github::{serve, util::workspace_dir, IntoResponse, SyncGithubError};
use toml::{map::Map, Value};

#[tokio::main]
async fn main() -> Result<(), SyncGithubError> {
    let out_file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("log.txt")
        .unwrap();

    tracing_subscriber::fmt().with_writer(out_file).init();

    let handle = tokio::spawn(async move {
        let secrets = load_env()?;

        let conn_string =
            secrets.get("LOCAL_DATABASE_URL").unwrap().as_str().unwrap();
        let github_token =
            secrets.get("GITHUB_TOKEN").unwrap().as_str().unwrap();

        serve(conn_string, github_token).await
    });

    let _ = handle.await.unwrap();

    return Ok(());
}

fn load_env() -> Result<Map<String, Value>, SyncGithubError> {
    let workspace_dir = workspace_dir();
    let secrets = std::fs::read_to_string(workspace_dir.join("Secrets.toml"))
        .into_response("failed to read Secrets.toml")?;

    let secrets = toml::from_str::<Map<String, Value>>(&secrets)
        .into_response("failed to parse Secrets.toml")?;

    Ok(secrets)
}
