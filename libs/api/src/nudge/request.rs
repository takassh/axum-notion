use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct PostNudgeParam {
    pub content: String,
}
