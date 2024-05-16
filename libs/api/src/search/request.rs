use cloudflare::models::text_generation::Message;
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct SearchParam {
    pub prompt: String,
    pub history: Vec<Message>,
}
