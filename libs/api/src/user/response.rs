use entity::prelude::*;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct GetUserResp {
    pub user: UserResp,
}

#[derive(Serialize, ToSchema)]
pub struct UserResp {
    pub created_at: String,
    pub updated_at: String,
}

impl From<UserEntity> for UserResp {
    fn from(value: UserEntity) -> Self {
        Self {
            created_at: value.created_at.to_string(),
            updated_at: value.updated_at.to_string(),
        }
    }
}
