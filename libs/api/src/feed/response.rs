use serde::Serialize;

#[derive(Serialize)]
pub struct Feed {
    pub contents: String,
}

#[derive(Serialize)]
pub struct GetFeedsResponse {
    pub feeds: Vec<Feed>,
}