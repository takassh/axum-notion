use serde::Serialize;

#[derive(Serialize)]
pub struct Page {
    pub contents: String,
}

#[derive(Serialize)]
pub struct GetPagesRespose {
    pub pages: Vec<Page>,
}

#[derive(Serialize)]
pub struct GetPageRespose {
    pub page: Option<Page>,
}
