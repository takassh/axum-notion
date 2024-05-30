pub mod implementation;

use futures_core::Stream;
use reqwest::Body;
use serde::{Deserialize, Serialize};

static LLAMA_3_8B_INSTRUCT: &str = "@cf/meta/llama-3-8b-instruct";
static HERMES_2_PRO_MISTRAL_7B: &str =
    "@hf/nousresearch/hermes-2-pro-mistral-7b";

pub trait TextGeneration {
    fn llama_3_8b_instruct(
        &self,
        request: TextGenerationRequest,
    ) -> impl std::future::Future<Output = anyhow::Result<TextGenerationResponse>>
           + Send;

    fn llama_3_8b_instruct_with_stream(
        self,
        request: TextGenerationRequest,
    ) -> impl Stream<Item = anyhow::Result<Vec<TextGenerationJsonResult>>> + Send;

    fn hermes_2_pro_mistral_7b(
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

#[derive(Debug, Serialize, Default)]
pub struct PromptRequest {
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<i32>,
}

#[derive(Debug, Serialize, Default)]
pub struct MessageRequest {
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct TextGenerationResponse {
    pub result: TextGenerationJsonResult,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TextGenerationJsonResult {
    pub response: String,
}

impl From<TextGenerationRequest> for Body {
    fn from(val: TextGenerationRequest) -> Self {
        let body = serde_json::to_string(&val).unwrap();
        Body::from(body)
    }
}
