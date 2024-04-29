use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

use crate::util::request::Pagination;

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct GetPagesParam {
    pub category: Option<String>,
    #[serde(flatten)]
    pub pagination: Pagination,
}
