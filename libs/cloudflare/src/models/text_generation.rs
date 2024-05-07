pub mod implementation;

use reqwest::Body;
use serde::{Deserialize, Serialize};

static LLAMA_3_8B_INSTRUCT: &str = "@cf/meta/llama-3-8b-instruct";

pub trait TextGeneration {
    fn llama_3_8b_instruct(
        &self,
        request: TextGenerationRequest,
    ) -> impl std::future::Future<Output = anyhow::Result<TextGenerationResponse>>
           + Send;
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum TextGenerationRequest {
    Prompt(PromptRequest),
    Message(MessageRequest),
}

#[derive(Debug, Serialize)]
pub struct PromptRequest {
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct MessageRequest {
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct TextGenerationResponse {
    pub result: TextGenerationJsonResult,
}

#[derive(Debug, Deserialize)]
pub struct TextGenerationJsonResult {
    pub response: String,
}

impl Into<Body> for TextGenerationRequest {
    fn into(self) -> Body {
        let body = serde_json::to_string(&self).unwrap();
        Body::from(body)
    }
}