use chrono::{TimeZone, Utc};
use sea_orm::{
    sea_query, ActiveValue, DatabaseConnection, EntityTrait, Iterable,
    PaginatorTrait, QueryFilter, QueryOrder,
};

use sea_orm::ColumnTrait;

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
            notion_database_id: value.notion_database_id,
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
            notion_database_id: ActiveValue::set(value.notion_database_id),
            created_at: ActiveValue::set(value.created_at.naive_utc()),
            updated_at: ActiveValue::set(
                value.updated_at.map(|f| f.naive_utc()),
            ),
            contents: ActiveValue::set(value.contents),
        }
    }
}

impl PageRepository {
    pub async fn find_paginate(
        &self,
        offset: u64,
        limit: u64,
        database_name: Option<String>,
    ) -> anyhow::Result<Vec<PageEntity>> {
        let database = NotionDatabase::find()
            .filter(notion_database::Column::Name.eq(database_name))
            .one(&self.db)
            .await?;

        let mut query = Page::find().order_by_desc(page::Column::CreatedAt);

        if let Some(database) = database {
            query =
                query.filter(page::Column::NotionDatabaseId.eq(database.id));
        }

        let pages = query.paginate(&self.db, limit).fetch_page(offset).await?;

        Ok(pages.into_iter().map(PageEntity::from).collect())
    }

    pub async fn find_all(&self) -> anyhow::Result<Vec<PageEntity>> {
        let pages = Page::find().all(&self.db).await?;

        Ok(pages.into_iter().map(PageEntity::from).collect())
    }

    pub async fn find_by_id(
        &self,
        id: String,
    ) -> anyhow::Result<Option<PageEntity>> {
        let page = page::Entity::find()
            .filter(Column::NotionPageId.eq(id))
            .one(&self.db)
            .await?;

        Ok(page.map(PageEntity::from))
    }

    pub async fn save(&self, page: PageEntity) -> anyhow::Result<()> {
        let mut page = page::ActiveModel::from(page);
        page.updated_at = ActiveValue::set(Some(Utc::now().naive_utc()));

        let _ = page::Entity::insert(page)
            .on_conflict(
                sea_query::OnConflict::column(page::Column::NotionPageId)
                    .update_columns(page::Column::iter())
                    .to_owned(),
            )
            .exec(&self.db)
            .await?;

        Ok(())
    }
}
