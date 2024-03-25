use reqwest::header::{HeaderMap, HeaderValue};
use serde::Serialize;
use toml::{map::Map, Value};

use anyhow::ensure;
use anyhow::Context as _;

#[derive(Clone, Debug)]
pub struct Client {
    base_url: String,
    headers: HeaderMap,
}

impl Client {
    pub fn new(
        token: String,
        config: &Map<String, Value>,
    ) -> anyhow::Result<Self> {
        let base_url = config
            .get("github")
            .context("failed to load github config")?
            .get("base_url")
            .context("failed to load base_url config")?;

        let mut headers = HeaderMap::new();
        headers.insert(
            "Accept",
            HeaderValue::from_str("application/vnd.github+json").unwrap(),
        );
        headers.insert(
            "Authorization",
            HeaderValue::from_str(format!("Bearer {}", token).as_str())
                .unwrap(),
        );
        headers.insert(
            "X-GitHub-Api-Version",
            HeaderValue::from_static("2022-11-28"),
        );
        headers.insert(
            "User-Agent",
            HeaderValue::from_str("Takassh-Rust-App").unwrap(),
        );

        Ok(Self {
            base_url: base_url.as_str().unwrap().to_string(),
            headers,
        })
    }

    pub async fn get<T: Serialize + ?Sized>(
        &self,
        path: &str,
        query: &T,
    ) -> anyhow::Result<(String, HeaderMap)> {
        let client = reqwest::Client::new();

        let response = client
            .get(format!("{}/{}", self.base_url, path))
            .headers(self.headers.clone())
            .query(query)
            .send()
            .await?;

        let status_code = response.status();
        let headers = response.headers().clone();

        let text = response.text().await?;

        ensure!(status_code.is_success(), "status code: {}", status_code);

        Ok((text, headers))
    }
}
