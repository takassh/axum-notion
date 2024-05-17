use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct PageResp {
    pub contents: String,
}

#[derive(Serialize, ToSchema)]
pub struct GetPagesResp {
    pub pages: Vec<PageResp>,
}

#[derive(Serialize, ToSchema)]
pub struct GetPageResp {
    pub page: Option<PageResp>,
}
