use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

use crate::request::Pagination;

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct GetPagesParam {
    pub category: Option<String>,
    #[serde(flatten)]
    pub pagination: Pagination,
}

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct GenerateCoverImageParam {
    pub prompt: String,
}

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct GenerateSummarizeParam {
    pub text: String,
}
