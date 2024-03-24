use chrono::Utc;
use sea_orm::{
    sea_query, DatabaseConnection, EntityTrait, IntoActiveModel, Iterable,
};

use entities::block::{self, Column};
use sea_orm::{ColumnTrait, QueryFilter};

use crate::{IntoResponse, Response};

#[derive(Clone, Debug)]
pub struct BlockRepository {
    db: DatabaseConnection,
}

impl BlockRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl BlockRepository {
    pub async fn find_all(
        &self,
    ) -> Response<Vec<block::Model>> {
        block::Entity::find().all(&self.db).await.into_response("find all")
    }

    pub async fn find_by_notion_page_id(
        &self,
        id: String,
    ) -> Response<Option<block::Model>> {
        block::Entity::find()
            .filter(Column::NotionPageId.eq(id))
            .one(&self.db)
            .await
            .into_response("find by notion page id")
    }

    pub async fn save(
        &self,
        mut block: block::Model,
    ) -> Response<()> {
        block.updated_at = Some(Utc::now().naive_utc());
        let _ = block::Entity::insert(block.into_active_model())
            .on_conflict(
                sea_query::OnConflict::column(block::Column::NotionPageId)
                    .update_columns(block::Column::iter())
                    .to_owned(),
            )
            .exec(&self.db)
            .await
            .into_response("save")?;

        Ok(())
    }
}
