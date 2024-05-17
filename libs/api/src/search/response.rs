use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct SearchResp {
    pub answer: String,
    pub session: String,
}
