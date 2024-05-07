use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

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
