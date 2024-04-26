use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Body, Response};

use anyhow::ensure;

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
}
