pub mod implementation;

use bytes::Bytes;
use reqwest::Body;
use serde::Serialize;

static STABLE_DIFFUSION_XL_LIGHTNING: &str =
    "@cf/bytedance/stable-diffusion-xl-lightning";

pub trait TextToImage {
    fn stable_diffusion_xl_lightning(
        &self,
        request: TextToImageRequest,
    ) -> impl std::future::Future<Output = anyhow::Result<Bytes>> + Send;
}

#[derive(Debug, Serialize, Default)]
pub struct TextToImageRequest {
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<Image>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mask: Option<Mask>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_steps: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strength: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guidance: Option<f32>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Image {
    String(String),
    Array(Vec<f32>),
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Mask {
    String(String),
    Array(Vec<f32>),
}

impl Into<Body> for TextToImageRequest {
    fn into(self) -> Body {
        let body = serde_json::to_string(&self).unwrap();
        Body::from(body)
    }
}
