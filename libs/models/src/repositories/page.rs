use chrono::Utc;
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, EntityTrait};

use crate::{entities::page, ModelsError};

use super::PageRepository;

impl PageRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl PageRepository {
    pub async fn find_all(&self) -> Result<Vec<page::Model>, ModelsError> {
        page::Entity::find()
            .all(&self.db)
            .await
            .map_err(|e| ModelsError::RepositoryError { source: e })
    }

    pub async fn find_by_id(&self, id: String) -> Result<Option<page::Model>, ModelsError> {
        page::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| ModelsError::RepositoryError { source: e })
    }

    pub async fn insert(&self, mut page: page::ActiveModel) -> Result<(), ModelsError> {
        page.created_at = ActiveValue::set(Utc::now().naive_utc());
        page.updated_at = ActiveValue::set(Some(Utc::now().naive_utc()));
        let _ = page
            .insert(&self.db)
            .await
            .map_err(|e| ModelsError::RepositoryError { source: e })?;

        Ok(())
    }

    pub async fn update(&self, mut page: page::ActiveModel) -> Result<(), ModelsError> {
        page.updated_at = ActiveValue::set(Some(Utc::now().naive_utc()));
        let _ = page
            .update(&self.db)
            .await
            .map_err(|e| ModelsError::RepositoryError { source: e })?;

        Ok(())
    }

    pub async fn delete_by_id(&self, id: String) -> Result<(), ModelsError> {
        let _ = page::Entity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(|e| ModelsError::RepositoryError { source: e })?;

        Ok(())
    }
}
