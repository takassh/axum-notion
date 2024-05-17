use chrono::{NaiveDateTime, Utc};
use sea_orm::{
    prelude::Uuid, sea_query, ActiveValue, DatabaseConnection, EntityTrait,
    Iterable,
};

use crate::active_models::{prelude::*, *};
use entity::prelude::*;

#[derive(Clone, Debug)]
pub struct PromptSessionRepository {
    db: DatabaseConnection,
}

impl PromptSessionRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl From<prompt_session::Model> for PromptSessionEntity {
    fn from(value: prompt_session::Model) -> Self {
        PromptSessionEntity {
            id: value.id,
            user_id: value.user_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl From<PromptSessionEntity> for prompt_session::ActiveModel {
    fn from(value: PromptSessionEntity) -> Self {
        Self {
            id: if value.id == String::default() {
                ActiveValue::Set(Uuid::new_v4().to_string())
            } else {
                ActiveValue::Set(value.id)
            },
            user_id: ActiveValue::Set(value.user_id),
            created_at: if value.created_at == NaiveDateTime::default() {
                ActiveValue::Set(Utc::now().naive_utc())
            } else {
                ActiveValue::Set(value.created_at)
            },
            updated_at: ActiveValue::Set(Utc::now().naive_utc()),
        }
    }
}

impl PromptSessionRepository {
    pub async fn find_all(&self) -> anyhow::Result<Vec<PromptSessionEntity>> {
        let prompt_sessions = PromptSession::find().all(&self.db).await?;

        Ok(prompt_sessions
            .into_iter()
            .map(PromptSessionEntity::from)
            .collect())
    }

    pub async fn save(
        &self,
        prompt_session: PromptSessionEntity,
    ) -> anyhow::Result<String> {
        let prompt_session = prompt_session::ActiveModel::from(prompt_session);

        let result = prompt_session::Entity::insert(prompt_session)
            .on_conflict(
                sea_query::OnConflict::column(prompt_session::Column::Id)
                    .update_columns(prompt_session::Column::iter())
                    .to_owned(),
            )
            .exec(&self.db)
            .await?;

        Ok(result.last_insert_id)
    }

    pub async fn delete(
        &self,
        prompt_session_id: String,
    ) -> anyhow::Result<()> {
        prompt_session::Entity::delete(prompt_session::ActiveModel {
            id: ActiveValue::Set(prompt_session_id),
            ..Default::default()
        })
        .exec(&self.db)
        .await?;

        Ok(())
    }
}
