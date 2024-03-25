use chrono::{TimeZone, Utc};
use sea_orm::{
    sea_query, ActiveValue, DatabaseConnection, EntityTrait, Iterable,
    QueryFilter,
};

use sea_orm::ColumnTrait;

use crate::IntoResponse;
use crate::Response;

use crate::active_models::{prelude::*, *};
use entity::prelude::*;

use self::page::Column;

#[derive(Clone, Debug)]
pub struct PageRepository {
    db: DatabaseConnection,
}

impl PageRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl From<page::Model> for PageEntity {
    fn from(value: page::Model) -> Self {
        Self {
            notion_page_id: value.notion_page_id,
            created_at: value.created_at.and_utc(),
            updated_at: value.updated_at.map(|f| Utc.from_utc_datetime(&f)),
            contents: value.contents,
        }
    }
}

impl From<PageEntity> for page::ActiveModel {
    fn from(value: PageEntity) -> Self {
        Self {
            notion_page_id: ActiveValue::set(value.notion_page_id),
            created_at: ActiveValue::set(value.created_at.naive_utc()),
            updated_at: ActiveValue::set(
                value.updated_at.map(|f| f.naive_utc()),
            ),
            contents: ActiveValue::set(value.contents),
        }
    }
}

impl PageRepository {
    pub async fn find_all(&self) -> Response<Vec<PageEntity>> {
        let pages =
            Page::find().all(&self.db).await.into_response("find all")?;

        Ok(pages.into_iter().map(PageEntity::from).collect())
    }

    pub async fn find_by_id(&self, id: String) -> Response<Option<PageEntity>> {
        let page = page::Entity::find()
            .filter(Column::NotionPageId.eq(id))
            .one(&self.db)
            .await
            .into_response("find by id")?;

        Ok(page.map(PageEntity::from))
    }

    pub async fn save(&self, page: PageEntity) -> Response<()> {
        let mut page = page::ActiveModel::from(page);
        page.updated_at = ActiveValue::set(Some(Utc::now().naive_utc()));

        let _ = page::Entity::insert(page)
            .on_conflict(
                sea_query::OnConflict::column(page::Column::NotionPageId)
                    .update_columns(page::Column::iter())
                    .to_owned(),
            )
            .exec(&self.db)
            .await
            .into_response("save")?;

        Ok(())
    }
}
