use chrono::Utc;
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, EntityTrait};

use crate::{entities::block, ModelsError};

use super::BlockRepository;

impl BlockRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl BlockRepository {
    pub async fn find_all(&self) -> Result<Vec<block::Model>, ModelsError> {
        block::Entity::find()
            .all(&self.db)
            .await
            .map_err(|e| ModelsError::RepositoryError { source: e })
    }

    pub async fn find_by_id(&self, id: String) -> Result<Option<block::Model>, ModelsError> {
        block::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| ModelsError::RepositoryError { source: e })
    }

    pub async fn insert(&self, mut block: block::ActiveModel) -> Result<(), ModelsError> {
        block.created_at = ActiveValue::set(Utc::now().naive_utc());
        block.updated_at = ActiveValue::set(Some(Utc::now().naive_utc()));
        let _ = block
            .insert(&self.db)
            .await
            .map_err(|e| ModelsError::RepositoryError { source: e })?;

        Ok(())
    }

    pub async fn update(&self, mut block: block::ActiveModel) -> Result<(), ModelsError> {
        block.updated_at = ActiveValue::set(Some(Utc::now().naive_utc()));
        let _ = block
            .update(&self.db)
            .await
            .map_err(|e| ModelsError::RepositoryError { source: e })?;

        Ok(())
    }

    pub async fn delete_by_id(&self, id: String) -> Result<(), ModelsError> {
        let _ = block::Entity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(|e| ModelsError::RepositoryError { source: e })?;

        Ok(())
    }
}
