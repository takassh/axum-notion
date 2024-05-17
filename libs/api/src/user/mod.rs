use std::sync::Arc;

use anyhow::anyhow;
use axum::{extract::State, Extension, Json};
use entity::user::User;

use crate::{
    auth::Claims,
    response::{ApiResponse, IntoApiResponse},
    ApiState,
};

use self::response::{GetUserResp, UserResp};
pub mod response;

/// Get user
#[utoipa::path(
    get,
    path = "/user",
    responses(
        (status = 200, description = "Get user successfully", body = [GetUserResponse])
    )
)]
pub async fn get_user(
    Extension(ref claims): Extension<Claims>,
    State(state): State<Arc<ApiState>>,
) -> ApiResponse<Json<GetUserResp>> {
    let user = state
        .repo
        .user
        .find_by_sub(&claims.sub)
        .await
        .into_response("502-013")?;

    let Some(user) = user else {
        let id = state
            .repo
            .user
            .save(User {
                sub: claims.sub.clone(),
                ..Default::default()
            })
            .await
            .into_response("502-013")?;

        let user = state.repo.user.find_by_id(id).await;

        let Ok(Some(user)) = user else {
            return Err(anyhow!("failed to get user. id: {}", id))
                .into_response("502-013");
        };

        return Ok(Json(GetUserResp {
            user: UserResp::from(user),
        }));
    };

    Ok(Json(GetUserResp {
        user: UserResp::from(user),
    }))
}
