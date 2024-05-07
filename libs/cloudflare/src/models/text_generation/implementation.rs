use anyhow::Context;

use crate::models::{text_generation::LLAMA_3_8B_INSTRUCT, Models};

use super::TextGeneration;

impl TextGeneration for Models {
    async fn llama_3_8b_instruct(
        &self,
        request: super::TextGenerationRequest,
    ) -> anyhow::Result<super::TextGenerationResponse> {
        let text = self.string_response(request, LLAMA_3_8B_INSTRUCT).await?;

        let response =
            serde_json::from_str(&text).context("failed to parse response")?;

        Ok(response)
    }
}
