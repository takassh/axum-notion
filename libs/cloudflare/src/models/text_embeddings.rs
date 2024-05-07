pub mod implementation;

use reqwest::Body;
use serde::{Deserialize, Serialize};

static BGE_BASE_EN_V1_5: &str = "@cf/baai/bge-base-en-v1.5";
static BGE_LARGE_EN_V1_5: &str = "@cf/baai/bge-large-en-v1.5";
static BGE_SMALL_EN_V1_5: &str = "@cf/baai/bge-small-en-v1.5";

pub trait TextEmbeddings {
    fn bge_base_en_v1_5(
        &self,
        request: TextEmbeddingsRequest,
    ) -> impl std::future::Future<Output = anyhow::Result<TextEmbeddingsResponse>>
           + Send;
    fn bge_large_en_v1_5(
        &self,
        request: TextEmbeddingsRequest,
    ) -> impl std::future::Future<Output = anyhow::Result<TextEmbeddingsResponse>>
           + Send;
    fn bge_small_en_v1_5(
        &self,
        request: TextEmbeddingsRequest,
    ) -> impl std::future::Future<Output = anyhow::Result<TextEmbeddingsResponse>>
           + Send;
}

#[derive(Debug, Serialize)]
pub struct TextEmbeddingsRequest {
    pub text: StringOrArray,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum StringOrArray {
    String(String),
    Array(Vec<String>),
}

#[derive(Debug, Deserialize)]
pub struct TextEmbeddingsResponse {
    pub result: TextEmbeddingsResult,
}

#[derive(Debug, Deserialize)]
pub struct TextEmbeddingsResult {
    pub shape: Vec<f32>,
    pub data: Vec<Vec<f32>>,
}

impl From<TextEmbeddingsRequest> for Body {
    fn from(val: TextEmbeddingsRequest) -> Self {
        let body = serde_json::to_string(&val).unwrap();
        Body::from(body)
    }
}

impl From<&str> for StringOrArray {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl From<Vec<&str>> for StringOrArray {
    fn from(value: Vec<&str>) -> Self {
        Self::Array(value.iter().map(|s| s.to_string()).collect())
    }
}
