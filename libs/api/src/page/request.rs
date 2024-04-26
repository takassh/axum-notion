use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::util::request::Pagination;

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct GetPagesParam {
    pub category: Option<String>,
    #[serde(flatten)]
    pub pagination: Pagination,
}

#[derive(Serialize, Deserialize, ToSchema, IntoParams)]
pub struct GenerateCoverImageRequest {
    pub prompt: String,
}
