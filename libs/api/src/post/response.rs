use entity::post::Category;
use serde::Serialize;

#[derive(Serialize)]
pub struct PostResp {
    pub category: Category,
    pub contents: String,
}

#[derive(Serialize)]
pub struct GetPostsResp {
    pub posts: Vec<PostResp>,
}
