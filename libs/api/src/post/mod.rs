use axum::{extract::State, Json};
use repository::Repository;

mod request;
pub mod response;

use crate::response::{ApiResponse, IntoApiResponse};

use self::response::{GetPostsResp, PostResp};

pub async fn get_posts(
    State(repo): State<Repository>,
) -> ApiResponse<Json<GetPostsResp>> {
    let posts = repo.post.find_all().await.into_response("502-007")?;

    let response = Json(GetPostsResp {
        posts: posts
            .into_iter()
            .map(|post| PostResp {
                category: post.category,
                contents: post.contents.unwrap_or_default(),
            })
            .collect(),
    });

    Ok(response)
}
