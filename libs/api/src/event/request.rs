use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

use crate::request::Pagination;

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct GetEventsParam {
    #[serde(flatten)]
    pub pagination: Pagination,
}
