use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct ResponsePage {
    pub contents: String,
}

#[derive(Serialize, ToSchema)]
pub struct GetPagesResponse {
    pub pages: Vec<ResponsePage>,
}

#[derive(Serialize, ToSchema)]
pub struct GetPageResponse {
    pub page: Option<ResponsePage>,
}
