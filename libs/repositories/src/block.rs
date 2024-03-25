use chrono::Utc;
use sea_orm::{
    sea_query, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveValue,
    Iterable,
};

use sea_orm::QueryFilter;

use crate::active_models::{prelude::*, *};
use crate::{IntoResponse, Response};
use chrono::TimeZone;
use entity::prelude::*;

use self::block::Column;

#[derive(Clone, Debug)]
pub struct BlockRepository {
    db: DatabaseConnection,
}

impl BlockRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl From<block::Model> for BlockEntity {
    fn from(value: block::Model) -> Self {
        Self {
            notion_page_id: value.notion_page_id,
            updated_at: value.updated_at.map(|f| Utc.from_utc_datetime(&f)),
            contents: value.contents,
        }
    }
}

impl From<BlockEntity> for block::ActiveModel {
    fn from(value: BlockEntity) -> Self {
        Self {
            notion_page_id: value.notion_page_id.into_active_value(),
            updated_at: value
                .updated_at
                .map(|f| f.naive_utc())
                .into_active_value(),
            contents: value.contents.into_active_value(),
        }
    }
}

impl BlockRepository {
    pub async fn find_all(&self) -> Response<Vec<BlockEntity>> {
        let blocks = Block::find()
            .all(&self.db)
            .await
            .into_response("find all")?;
        Ok(blocks.into_iter().map(BlockEntity::from).collect())
    }

    pub async fn find_by_notion_page_id(
        &self,
        id: String,
    ) -> Response<Option<BlockEntity>> {
        let block = block::Entity::find()
            .filter(Column::NotionPageId.eq(id))
            .one(&self.db)
            .await
            .into_response("find by notion page id")?;

        Ok(block.map(BlockEntity::from))
    }

    pub async fn save(&self, block: BlockEntity) -> Response<()> {
        let mut block = block::ActiveModel::from(block);
        block.updated_at = Some(Utc::now().naive_utc()).into_active_value();

        let _ = block::Entity::insert(block)
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
