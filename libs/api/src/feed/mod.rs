use axum::{extract::State, Json};
use repository::Repository;

mod request;
pub mod response;

use crate::response::{ApiResponse, IntoApiResponse};

use self::response::{GetPostsResponse, Post};

pub async fn get_feeds(
    State(repo): State<Repository>,
) -> ApiResponse<Json<GetPostsResponse>> {
    let feeds = repo.feed.find_all().await.into_response("502-007")?;

    let response = Json(GetPostsResponse {
        posts: feeds
            .into_iter()
            .map(|post| Post {
                category: post.category,
                contents: "".to_string(),
            })
            .collect(),
    });

    Ok(response)
}
