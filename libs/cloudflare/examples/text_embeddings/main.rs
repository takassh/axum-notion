use anyhow::Context;
use cloudflare::models::{
    self,
    text_embeddings::{TextEmbeddings, TextEmbeddingsRequest},
};
use toml::{map::Map, Value};
use util::workspace_dir;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let secrets = load_env()?;

    let token = secrets.get("CLOUDFLARE_TOKEN").unwrap().as_str().unwrap();
    let account_id = secrets
        .get("CLOUDFLARE_ACCOUNT_ID")
        .unwrap()
        .as_str()
        .unwrap();

    let models = models::Models::new(account_id, token);

    let result = models
        .bge_base_en_v1_5(TextEmbeddingsRequest {
            text: "Hello, world!".into(),
        })
        .await?;

    println!("{:?}", result);

    Ok(())
}

fn load_env() -> anyhow::Result<Map<String, Value>> {
    let workspace_dir = workspace_dir();
    let secrets =
        std::fs::read_to_string(workspace_dir.join("Secrets.dev.toml"))
            .context("failed to read Secrets.dev.toml")?;

    toml::from_str::<Map<String, Value>>(&secrets)
        .context("failed to parse Secrets.dev.toml")
}
