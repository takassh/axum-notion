use serde::Serialize;

#[derive(Serialize)]
pub struct Page {
    pub contents: String,
}

#[derive(Serialize)]
pub struct GetPagesResponse {
    pub pages: Vec<Page>,
}

#[derive(Serialize)]
pub struct GetPageResponse {
    pub page: Option<Page>,
}
