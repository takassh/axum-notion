use anyhow::ensure;
use bytes::Bytes;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Body, Client,
};

pub mod text_embeddings;
pub mod text_generation;
pub mod text_to_image;
pub mod translation;

#[derive(Debug, Clone)]
pub struct Models {
    base_url: String,
    client: Client,
}

impl Models {
    pub fn new(account_id: &str, token: &str) -> Self {
        let base_url = format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/ai/run",
            account_id
        );
        let mut headers = HeaderMap::new();
        headers.insert("Accept", HeaderValue::from_str("*/*").unwrap());
        headers.insert(
            "Authorization",
            HeaderValue::from_str(format!("Bearer {}", token).as_str())
                .unwrap(),
        );

        let client = reqwest::ClientBuilder::new()
            .default_headers(headers)
            .build()
            .unwrap();

        Self { base_url, client }
    }

    async fn string_response<R: Into<Body>>(
        &self,
        request: R,
        model: &str,
    ) -> anyhow::Result<String> {
        let response = self
            .client
            .post(format!("{}/{}", self.base_url, model))
            .body(request)
            .send()
            .await?;

        let status_code = response.status();
        let text = response.text().await;

        ensure!(
            status_code.is_success(),
            "status code: {}, response: {:?}",
            status_code,
            text
        );

        Ok(text?)
    }

    async fn binary_response<R: Into<Body>>(
        &self,
        request: R,
        model: &str,
    ) -> anyhow::Result<Bytes> {
        let response = self
            .client
            .post(format!("{}/{}", self.base_url, model))
            .body(request)
            .send()
            .await?;

        let status_code = response.status();
        let bytes = response.bytes().await;

        ensure!(status_code.is_success(), "status code: {}", status_code);

        Ok(bytes?)
    }
}
