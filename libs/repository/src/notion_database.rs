use sea_orm::{
    sea_query, ActiveValue, DatabaseConnection, EntityTrait, Iterable,
};

use crate::active_models::{prelude::*, *};
use entity::prelude::*;

#[derive(Clone, Debug)]
pub struct NotionDatabaseRepository {
    db: DatabaseConnection,
}

impl NotionDatabaseRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl From<notion_database::Model> for NotionDatabaseEntity {
    fn from(value: notion_database::Model) -> Self {
        Self {
            id: value.id,
            name: value.name,
            created_at: value.created_at.and_utc(),
        }
    }
}

impl From<NotionDatabaseEntity> for notion_database::ActiveModel {
    fn from(value: NotionDatabaseEntity) -> Self {
        Self {
            id: ActiveValue::set(value.id),
            name: ActiveValue::set(value.name),
            created_at: ActiveValue::set(value.created_at.naive_utc()),
        }
    }
}

impl NotionDatabaseRepository {
    pub async fn find_all(&self) -> anyhow::Result<Vec<NotionDatabaseEntity>> {
        let notion_databases = NotionDatabase::find().all(&self.db).await?;

        Ok(notion_databases
            .into_iter()
            .map(NotionDatabaseEntity::from)
            .collect())
    }

    pub async fn save(
        &self,
        notion_database: NotionDatabaseEntity,
    ) -> anyhow::Result<()> {
        let notion_database =
            notion_database::ActiveModel::from(notion_database);

        let _ = notion_database::Entity::insert(notion_database)
            .on_conflict(
                sea_query::OnConflict::column(notion_database::Column::Id)
                    .update_columns(notion_database::Column::iter())
                    .to_owned(),
            )
            .exec(&self.db)
            .await?;

        Ok(())
    }
}
