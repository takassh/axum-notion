use chrono::Utc;
use sea_orm::{sea_query, DatabaseConnection, EntityTrait, Iterable, QueryFilter};

use entities::page::{self, Column};
use sea_orm::ColumnTrait;
use sea_orm::IntoActiveModel;

use crate::RepositoriesError;

#[derive(Clone, Debug)]
pub struct PageRepository {
    db: DatabaseConnection,
}

impl PageRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl PageRepository {
    pub async fn find_all(&self) -> Result<Vec<page::Model>, RepositoriesError> {
        page::Entity::find()
            .all(&self.db)
            .await
            .map_err(|e| RepositoriesError::FailedToQuery { source: e })
    }

    pub async fn find_by_id(&self, id: String) -> Result<Option<page::Model>, RepositoriesError> {
        page::Entity::find()
            .filter(Column::NotionPageId.eq(id))
            .one(&self.db)
            .await
            .map_err(|e| RepositoriesError::FailedToQuery { source: e })
    }

    pub async fn save(&self, mut page: page::Model) -> Result<(), RepositoriesError> {
        page.updated_at = Some(Utc::now().naive_utc());
        let _ = page::Entity::insert(page.into_active_model())
            .on_conflict(
                sea_query::OnConflict::column(page::Column::NotionPageId)
                    .update_columns(page::Column::iter())
                    .to_owned(),
            )
            .exec(&self.db)
            .await
            .map_err(|e| RepositoriesError::FailedToSave { source: e })?;

        Ok(())
    }
}
