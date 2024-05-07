pub mod implementation;

use reqwest::Body;
use serde::{Deserialize, Serialize};

static M2M100_1_2B: &str = "@cf/meta/m2m100-1.2b";

pub trait Translation {
    fn m2m100_1_2b(
        &self,
        request: TranslationRequest,
    ) -> impl std::future::Future<Output = anyhow::Result<TranslationResponse>> + Send;
}

#[derive(Debug, Serialize)]
pub struct TranslationRequest {
    pub text: String,
    pub source_lang: String,
    pub target_lang: String,
}

#[derive(Debug, Deserialize)]
pub struct TranslationResponse {
    pub result: TranslationResult,
}

#[derive(Debug, Deserialize)]
pub struct TranslationResult {
    pub translated_text: String,
}

impl From<TranslationRequest> for Body {
    fn from(val: TranslationRequest) -> Self {
        let body = serde_json::to_string(&val).unwrap();
        Body::from(body)
    }
}
