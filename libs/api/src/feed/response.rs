use entity::post::Category;
use serde::Serialize;

#[derive(Serialize)]
pub struct Post {
    pub category: Category,
    pub contents: String,
}

#[derive(Serialize)]
pub struct GetPostsResponse {
    pub posts: Vec<Post>,
}
