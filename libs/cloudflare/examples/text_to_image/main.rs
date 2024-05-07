use std::{fs::File, io::Write};

use anyhow::Context;
use cloudflare::models::{
    self,
    text_to_image::{TextToImage, TextToImageRequest},
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
        .stable_diffusion_xl_lightning(TextToImageRequest {
            prompt: "Hello World!".to_string(),
            ..Default::default()
        })
        .await?;

    File::create("image.jpg")
        .unwrap()
        .write_all(&result)
        .unwrap();

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
