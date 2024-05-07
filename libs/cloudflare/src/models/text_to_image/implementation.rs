use bytes::Bytes;

use crate::models::Models;

use super::{TextToImage, STABLE_DIFFUSION_XL_LIGHTNING};

impl TextToImage for Models {
    async fn stable_diffusion_xl_lightning(
        &self,
        request: super::TextToImageRequest,
    ) -> anyhow::Result<Bytes> {
        let bytes = self
            .binary_response(request, STABLE_DIFFUSION_XL_LIGHTNING)
            .await?;

        Ok(bytes)
    }
}
