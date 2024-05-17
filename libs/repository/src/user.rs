use chrono::{NaiveDateTime, Utc};
use sea_orm::{
    entity::*, ActiveValue, DatabaseConnection, EntityTrait, QueryFilter,
};

use crate::active_models::{prelude::*, *};
use entity::prelude::*;

#[derive(Clone, Debug)]
pub struct UserRepository {
    db: DatabaseConnection,
}

impl UserRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl From<user::Model> for UserEntity {
    fn from(value: user::Model) -> Self {
        UserEntity {
            id: value.id,
            sub: value.sub,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl From<UserEntity> for user::ActiveModel {
    fn from(value: UserEntity) -> Self {
        Self {
            id: {
                if value.id == i32::default() {
                    ActiveValue::not_set()
                } else {
                    ActiveValue::Set(value.id)
                }
            },
            sub: ActiveValue::Set(value.sub),
            created_at: if value.created_at == NaiveDateTime::default() {
                ActiveValue::Set(Utc::now().naive_utc())
            } else {
                ActiveValue::Set(value.created_at)
            },
            updated_at: ActiveValue::Set(Utc::now().naive_utc()),
        }
    }
}

impl UserRepository {
    pub async fn find_by_id(
        &self,
        id: i32,
    ) -> anyhow::Result<Option<UserEntity>> {
        let user = User::find_by_id(id).one(&self.db).await?;

        Ok(user.map(UserEntity::from))
    }

    pub async fn find_by_sub(
        &self,
        sub: &str,
    ) -> anyhow::Result<Option<UserEntity>> {
        let user = User::find()
            .filter(user::Column::Sub.eq(sub))
            .one(&self.db)
            .await?;

        Ok(user.map(UserEntity::from))
    }

    pub async fn find_all(&self) -> anyhow::Result<Vec<UserEntity>> {
        let users = User::find().all(&self.db).await?;
        Ok(users.into_iter().map(UserEntity::from).collect())
    }

    pub async fn save(&self, user: UserEntity) -> anyhow::Result<i32> {
        let user = user::ActiveModel::from(user).save(&self.db).await?;
        Ok(user.id.unwrap())
    }

    pub async fn delete(&self, user_id: i32) -> anyhow::Result<()> {
        user::Entity::delete(user::ActiveModel {
            id: ActiveValue::Set(user_id),
            ..Default::default()
        })
        .exec(&self.db)
        .await?;

        Ok(())
    }
}
