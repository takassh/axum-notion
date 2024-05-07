use anyhow::Context;

use crate::models::{
    text_embeddings::{BGE_BASE_EN_V1_5, BGE_LARGE_EN_V1_5, BGE_SMALL_EN_V1_5},
    Models,
};

use super::{TextEmbeddings, TextEmbeddingsRequest, TextEmbeddingsResponse};

impl TextEmbeddings for Models {
    async fn bge_base_en_v1_5(
        &self,
        request: TextEmbeddingsRequest,
    ) -> anyhow::Result<TextEmbeddingsResponse> {
        let text = self.string_response(request, BGE_BASE_EN_V1_5).await?;

        let response =
            serde_json::from_str(&text).context("failed to parse response")?;

        Ok(response)
    }

    async fn bge_large_en_v1_5(
        &self,
        request: TextEmbeddingsRequest,
    ) -> anyhow::Result<TextEmbeddingsResponse> {
        let text = self.string_response(request, BGE_LARGE_EN_V1_5).await?;

        let response =
            serde_json::from_str(&text).context("failed to parse response")?;

        Ok(response)
    }

    async fn bge_small_en_v1_5(
        &self,
        request: TextEmbeddingsRequest,
    ) -> anyhow::Result<TextEmbeddingsResponse> {
        let text = self.string_response(request, BGE_SMALL_EN_V1_5).await?;

        let response =
            serde_json::from_str(&text).context("failed to parse response")?;

        Ok(response)
    }
}
