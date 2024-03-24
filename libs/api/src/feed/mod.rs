use axum::{extract::State, Json};
use repositories::Repository;

mod request;
pub mod response;

use crate::{ApiResponse, IntoApiResponse};

use self::response::{Feed, GetFeedsResponse};

pub async fn get_feeds(
    State(repo): State<Repository>,
) -> ApiResponse<Json<GetFeedsResponse>> {
    let feeds = repo.feed.find_all().await.into_response("502-007")?;

    let response = Json(GetFeedsResponse {
        feeds: feeds
            .into_iter()
            .map(|a| Feed {
                contents: a.contents,
            })
            .collect(),
    });

    Ok(response)
}
