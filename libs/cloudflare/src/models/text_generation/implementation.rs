use anyhow::Context;
use async_stream::stream;
use futures_core::Stream;

use crate::models::{text_generation::LLAMA_3_8B_INSTRUCT, Models};
use futures_util::StreamExt;

use super::{
    TextGeneration, TextGenerationJsonResult, HERMES_2_PRO_MISTRAL_7B,
};

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

    fn llama_3_8b_instruct_with_stream(
        self,
        request: super::TextGenerationRequest,
    ) -> impl Stream<Item = anyhow::Result<Vec<TextGenerationJsonResult>>> {
        stream! {
        let mut stream =
        self.stream_response(request, LLAMA_3_8B_INSTRUCT).await?;
                while let Some(s) = stream.next().await.transpose()? {
                    let data:Vec<TextGenerationJsonResult> = String::from_utf8(s.to_vec())?.split("data: ").flat_map(serde_json::from_str).collect();
                    yield Ok(data);
                }
            }
    }

    async fn hermes_2_pro_mistral_7b(
        &self,
        request: super::TextGenerationRequest,
    ) -> anyhow::Result<super::TextGenerationResponse> {
        let text = self
            .string_response(request, HERMES_2_PRO_MISTRAL_7B)
            .await?;

        let response =
            serde_json::from_str(&text).context("failed to parse response")?;

        Ok(response)
    }
}
