use serde::Serialize;

#[derive(Serialize)]
pub struct PostCodeResponse {
    pub result: String,
}
