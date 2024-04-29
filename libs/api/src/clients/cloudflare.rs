use axum::body::Bytes;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Body, Response};

use anyhow::{ensure, Context};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Clone, Debug)]
pub struct Client {
    base_url: String,
    id: String,
    headers: HeaderMap,
}

impl Client {
    pub fn new(
        token: String,
        id: String,
        base_url: String,
    ) -> anyhow::Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert("Accept", HeaderValue::from_str("*/*").unwrap());
        headers.insert(
            "Authorization",
            HeaderValue::from_str(format!("Bearer {}", token).as_str())
                .unwrap(),
        );

        Ok(Self {
            base_url,
            id,
            headers,
        })
    }

    pub async fn post<T: Into<Body>>(
        &self,
        path: &str,
        body: T,
    ) -> anyhow::Result<Response> {
        let client = reqwest::Client::new();

        let response = client
            .post(format!(
                "{}/client/v4/accounts/{}/{}",
                self.base_url, self.id, path
            ))
            .headers(self.headers.clone())
            .body(body)
            .send()
            .await?;

        let status_code = response.status();
        let _headers = response.headers().clone();

        ensure!(status_code.is_success(), "status code: {}", status_code);

        Ok(response)
    }

    pub async fn translate(
        &self,
        path: &str,
        body: TranslationRequest,
    ) -> anyhow::Result<String> {
        let response = self
            .post(
                path,
                serde_json::to_string(&body)
                    .context("failed to serialize body")?,
            )
            .await?;

        let text = response.text().await?;
        let response: TranslationResponse =
            serde_json::from_str(&text).context("failed to parse response")?;

        Ok(response.result.translated_text)
    }

    pub async fn generate_text(
        &self,
        path: &str,
        body: GenerateTextRequest,
    ) -> anyhow::Result<String> {
        let response = self
            .post(
                path,
                serde_json::to_string(&body)
                    .context("failed to serialize body")?,
            )
            .await?;

        let text = response.text().await?;
        let response: GenerateTextResponse =
            serde_json::from_str(&text).context("failed to parse response")?;

        Ok(response.result.response)
    }

    pub async fn generate_image(
        &self,
        path: &str,
        body: GenerateImageRequest,
    ) -> anyhow::Result<Bytes> {
        let response = self
            .post(
                path,
                serde_json::to_string(&body)
                    .context("failed to serialize body")?,
            )
            .await?;

        let image = response
            .bytes()
            .await
            .context("failed to get response bytes")?;

        Ok(image)
    }
}

#[derive(Serialize, Deserialize, ToSchema, IntoParams, Debug)]
pub struct TranslationRequest {
    pub source_lang: String,
    pub target_lang: String, // TODO: enum
    pub text: String,
}

#[derive(Serialize, Deserialize, ToSchema, IntoParams, Debug)]
pub struct GenerateTextRequest {
    pub stream: bool,
    pub messages: Vec<GenerateTextMessage>,
}

#[derive(Serialize, Deserialize, ToSchema, IntoParams, Debug)]
pub struct GenerateTextMessage {
    pub role: String, // TODO: enum
    pub content: String,
}

#[derive(Serialize, Deserialize, ToSchema, IntoParams, Debug)]
pub struct GenerateImageRequest {
    pub prompt: String,
}

#[derive(Serialize, Deserialize, ToSchema, IntoParams, Debug)]
pub struct GenerateImageFromText {
    pub text: String,
}

#[derive(Serialize, Deserialize, ToSchema, IntoParams, Debug)]
pub struct SummarizationRequest {
    pub input_text: String,
    pub max_length: u32,
}

////////////////////////////// Response //////////////////////////////
#[derive(Serialize, Deserialize, ToSchema, IntoParams, Debug)]
pub struct TranslationResponse {
    pub result: TranslationResult,
    pub success: bool,
    pub errors: Vec<String>,
    pub messages: Vec<String>,
}

#[derive(Serialize, Deserialize, ToSchema, IntoParams, Debug)]
pub struct TranslationResult {
    pub translated_text: String,
}

#[derive(Serialize, Deserialize, ToSchema, IntoParams, Debug)]
pub struct GenerateTextResponse {
    pub result: GenerateTextResult,
}

#[derive(Serialize, Deserialize, ToSchema, IntoParams, Debug)]
pub struct GenerateTextResult {
    pub response: String,
}
