use chrono::{TimeZone, Utc};
use sea_orm::{
    sea_query, ActiveValue, DatabaseConnection, EntityTrait, IntoActiveValue,
    Iterable, QueryOrder, QuerySelect,
};

use sea_orm::{ColumnTrait, QueryFilter};

use crate::active_models::{prelude::*, *};
use entity::prelude::*;

use self::event::Column;

#[derive(Clone, Debug)]
pub struct EventRepository {
    db: DatabaseConnection,
}

impl EventRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl From<event::Model> for EventEntity {
    fn from(value: event::Model) -> Self {
        Self {
            github_event_id: value.github_event_id,
            created_at: value.created_at.and_utc(),
            updated_at: value.updated_at.map(|f| Utc.from_utc_datetime(&f)),
            contents: value.contents,
        }
    }
}

impl From<EventEntity> for event::ActiveModel {
    fn from(value: EventEntity) -> Self {
        Self {
            github_event_id: value.github_event_id.into_active_value(),
            created_at: value.created_at.naive_utc().into_active_value(),
            updated_at: value
                .updated_at
                .map(|f| f.naive_utc())
                .into_active_value(),
            contents: value.contents.into_active_value(),
        }
    }
}

impl EventRepository {
    pub async fn find(
        &self,
        offset: u64,
        limit: u64,
    ) -> anyhow::Result<Vec<EventEntity>> {
        let events = Event::find()
            .order_by_desc(event::Column::CreatedAt)
            .offset(offset)
            .limit(limit)
            .all(&self.db)
            .await?;

        Ok(events.into_iter().map(EventEntity::from).collect())
    }

    pub async fn find_all(&self) -> anyhow::Result<Vec<EventEntity>> {
        let events = Event::find().all(&self.db).await?;

        Ok(events.into_iter().map(EventEntity::from).collect())
    }

    pub async fn find_by_event_id(
        &self,
        id: String,
    ) -> anyhow::Result<Option<EventEntity>> {
        let event = event::Entity::find()
            .filter(Column::GithubEventId.eq(id))
            .one(&self.db)
            .await?;

        Ok(event.map(EventEntity::from))
    }

    pub async fn save(&self, event: EventEntity) -> anyhow::Result<()> {
        let mut active_model = event::ActiveModel::from(event);
        active_model.updated_at =
            ActiveValue::set(Some(Utc::now().naive_utc()));

        let _ = event::Entity::insert(active_model)
            .on_conflict(
                sea_query::OnConflict::column(event::Column::GithubEventId)
                    .update_columns(event::Column::iter())
                    .to_owned(),
            )
            .exec(&self.db)
            .await?;

        Ok(())
    }
}
