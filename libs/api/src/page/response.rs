use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct Page {
    pub contents: String,
}

#[derive(Serialize, ToSchema)]
pub struct GetPagesResponse {
    pub pages: Vec<Page>,
}

#[derive(Serialize, ToSchema)]
pub struct GetPageResponse {
    pub page: Option<Page>,
}
