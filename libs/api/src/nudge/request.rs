use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct PostNudgeParam {
    pub page_id: String,
    pub content: String,
}
