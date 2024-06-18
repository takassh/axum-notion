pub mod implementation;

use std::collections::HashMap;

use futures_core::Stream;
use reqwest::Body;
use serde::{Deserialize, Serialize};

pub static LLAMA_3_8B_INSTRUCT: &str = "@cf/meta/llama-3-8b-instruct-awq";
pub static HERMES_2_PRO_MISTRAL_7B: &str =
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
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub model_parameters: Option<ModelParameters>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TextGenerationResponse {
    pub result: TextGenerationJsonResult,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TextGenerationJsonResult {
    pub response: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ToolCall {
    pub name: String,
    pub arguments: Option<HashMap<String, Option<String>>>,
}

#[derive(Serialize, Default)]
pub struct Tool {
    pub r#type: String,
    pub function: Function,
}

#[derive(Serialize, Default)]
pub struct Function {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Parameters>,
}

#[derive(Serialize, Default)]
pub struct Parameters {
    pub r#type: String,
    pub properties: HashMap<String, PropertyType>,
    pub required: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Default, Clone)]
pub struct ModelParameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<i32>, // from 0 to 5
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<i32>, // from 0 to 2
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<i32>, // from 1 to 50
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repetition_penalty: Option<i32>, // from 0 to 2
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<i32>, // from 0 to 2
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<i32>, // from 0 to 2
}

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum PropertyType {
    String,
    Number,
}

impl From<TextGenerationRequest> for Body {
    fn from(val: TextGenerationRequest) -> Self {
        let body = serde_json::to_string(&val).unwrap();
        Body::from(body)
    }
}
