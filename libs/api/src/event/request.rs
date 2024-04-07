use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

use crate::util::request::Pagination;

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct GetEventsParam {
    #[serde(flatten)]
    pub pagination: Pagination,
}
