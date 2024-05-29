use chrono::{NaiveDateTime, Utc};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, EntityTrait};

use crate::active_models::{prelude::*, *};
use entity::prelude::*;

#[derive(Clone, Debug)]
pub struct NudgeRepository {
    db: DatabaseConnection,
}

impl NudgeRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl From<nudge::Model> for NudgeEntity {
    fn from(value: nudge::Model) -> Self {
        NudgeEntity {
            id: value.id,
            created_at: value.created_at,
            content: value.content,
        }
    }
}

impl From<NudgeEntity> for nudge::ActiveModel {
    fn from(value: NudgeEntity) -> Self {
        Self {
            id: if value.id == i32::default() {
                ActiveValue::not_set()
            } else {
                ActiveValue::Set(value.id)
            },
            created_at: if value.created_at == NaiveDateTime::default() {
                ActiveValue::Set(Utc::now().naive_utc())
            } else {
                ActiveValue::Set(value.created_at)
            },
            content: ActiveValue::Set(value.content),
        }
    }
}

impl NudgeRepository {
    pub async fn find_all(&self) -> anyhow::Result<Vec<NudgeEntity>> {
        let nudges = Nudge::find().all(&self.db).await?;

        Ok(nudges.into_iter().map(NudgeEntity::from).collect())
    }

    pub async fn save(&self, nudge: NudgeEntity) -> anyhow::Result<i32> {
        let nudge = nudge::ActiveModel::from(nudge).save(&self.db).await?;
        let nudge_id = nudge.id.unwrap();

        Ok(nudge_id)
    }

    pub async fn delete(&self, nudge_id: i32) -> anyhow::Result<()> {
        nudge::Entity::delete(nudge::ActiveModel {
            id: ActiveValue::Set(nudge_id),
            ..Default::default()
        })
        .exec(&self.db)
        .await?;

        Ok(())
    }
}
