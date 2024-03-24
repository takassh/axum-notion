use reqwest::header::{HeaderMap, HeaderValue};
use serde::Serialize;
use toml::{map::Map, Value};

use crate::SyncGithubError;

#[derive(Clone, Debug)]
pub struct Client {
    base_url: String,
    headers: HeaderMap,
}

impl Client {
    pub fn new(
        token: String,
        config: &Map<String, Value>,
    ) -> Result<Self, SyncGithubError> {
        let base_url = config
            .get("github")
            .ok_or_else(|| SyncGithubError::FailedToInitGithubClient {
                message: "failed to load github config".to_string(),
            })?
            .get("base_url")
            .ok_or_else(|| SyncGithubError::FailedToInitGithubClient {
                message: "failed to load base_url config".to_string(),
            })?;

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
    ) -> Result<(String, HeaderMap), SyncGithubError> {
        let client = reqwest::Client::new();

        let response = client
            .get(format!("{}/{}", self.base_url, path))
            .headers(self.headers.clone())
            .query(query)
            .send()
            .await
            .map_err(|e| SyncGithubError::FailedToCallAPI { source: e })?;

        let status = response.status();
        let headers = response.headers().clone();

        let text = response
            .text()
            .await
            .map_err(|e| SyncGithubError::FailedToGetResponse { source: e })?;

        if !status.is_success() {
            return Err(SyncGithubError::FailedStatusCode {
                code: status,
                message: text,
            });
        }

        Ok((text, headers))
    }
}
