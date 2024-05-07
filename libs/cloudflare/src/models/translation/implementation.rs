use anyhow::Context;

use crate::models::Models;

use super::{Translation, TranslationResponse, M2M100_1_2B};

impl Translation for Models {
    async fn m2m100_1_2b(
        &self,
        request: super::TranslationRequest,
    ) -> anyhow::Result<TranslationResponse> {
        let text = self.string_response(request, M2M100_1_2B).await?;

        let response =
            serde_json::from_str(&text).context("failed to parse response")?;

        Ok(response)
    }
}
