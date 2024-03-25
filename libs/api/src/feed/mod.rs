use axum::{extract::State, Json};
use repository::Repository;

mod request;
pub mod response;

use crate::response::{ApiResponse, IntoApiResponse};

use self::response::{GetPostsResponse, Post};

pub async fn get_posts(
    State(repo): State<Repository>,
) -> ApiResponse<Json<GetPostsResponse>> {
    let posts = repo.post.find_all().await.into_response("502-007")?;

    let response = Json(GetPostsResponse {
        posts: posts
            .into_iter()
            .map(|post| Post {
                category: post.category,
                contents: "".to_string(),
            })
            .collect(),
    });

    Ok(response)
}
