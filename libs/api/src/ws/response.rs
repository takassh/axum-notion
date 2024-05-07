use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

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
