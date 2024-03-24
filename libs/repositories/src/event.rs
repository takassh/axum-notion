use chrono::Utc;
use sea_orm::{
    sea_query, DatabaseConnection, EntityTrait, IntoActiveModel, Iterable,
};

use entities::event::{self, Column};
use sea_orm::{ColumnTrait, QueryFilter};

use crate::{IntoResponse, Response};

#[derive(Clone, Debug)]
pub struct EventRepository {
    db: DatabaseConnection,
}

impl EventRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl EventRepository {
    pub async fn find_all(
        &self,
    ) -> Response<Vec<event::Model>> {
        event::Entity::find().all(&self.db).await.into_response("find all")
    }

    pub async fn find_by_event_id(
        &self,
        id: String,
    ) -> Response<Option<event::Model>> {
        event::Entity::find()
            .filter(Column::EventId.eq(id))
            .one(&self.db)
            .await
            .into_response("find by event id")
    }

    pub async fn save(
        &self,
        mut event: event::Model,
    ) -> Response<()> {
        event.updated_at = Some(Utc::now().naive_utc());
        let _ = event::Entity::insert(event.into_active_model())
            .on_conflict(
                sea_query::OnConflict::column(event::Column::EventId)
                    .update_columns(event::Column::iter())
                    .to_owned(),
            )
            .exec(&self.db)
            .await
            .into_response("save")?;

        Ok(())
    }
}
