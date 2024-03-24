use chrono::Utc;
use sea_orm::{
    sea_query, DatabaseConnection, EntityTrait, IntoActiveModel, Iterable,
};

use entities::event::{self, Column};
use sea_orm::{ColumnTrait, QueryFilter};

use crate::RepositoriesError;

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
    ) -> Result<Vec<event::Model>, RepositoriesError> {
        event::Entity::find()
            .all(&self.db)
            .await
            .map_err(|e| RepositoriesError::FailedToQuery { source: e })
    }

    pub async fn find_by_event_id(
        &self,
        id: String,
    ) -> Result<Option<event::Model>, RepositoriesError> {
        event::Entity::find()
            .filter(Column::EventId.eq(id))
            .one(&self.db)
            .await
            .map_err(|e| RepositoriesError::FailedToQuery { source: e })
    }

    pub async fn save(
        &self,
        mut event: event::Model,
    ) -> Result<(), RepositoriesError> {
        event.updated_at = Some(Utc::now().naive_utc());
        let _ = event::Entity::insert(event.into_active_model())
            .on_conflict(
                sea_query::OnConflict::column(event::Column::EventId)
                    .update_columns(event::Column::iter())
                    .to_owned(),
            )
            .exec(&self.db)
            .await
            .map_err(|e| RepositoriesError::FailedToSave { source: e })?;

        Ok(())
    }
}
