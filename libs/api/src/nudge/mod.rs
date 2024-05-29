use std::sync::Arc;

use axum::{extract::State, Json};
use entity::nudge::Nudge;
use request::PostNudgeParam;

use crate::{
    response::{ApiResponse, IntoApiResponse},
    ApiState,
};

pub mod request;

/// Nudge
#[utoipa::path(
    post,
    path = "/nudge",
    responses(
        (status = 200, description = "Nudge successfully",body = [PostNudgeParam])
    )
)]
pub async fn post_nudge(
    State(state): State<Arc<ApiState>>,
    Json(params): Json<PostNudgeParam>,
) -> ApiResponse<()> {
    state
        .repo
        .nudge
        .save(Nudge {
            content: params.content,
            ..Default::default()
        })
        .await
        .into_response("502-020")?;
    Ok(())
}
