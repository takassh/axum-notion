use chrono::{TimeZone, Utc};
use entity::page::ParentType;
use sea_orm::{
    sea_query, ActiveValue, DatabaseConnection, EntityTrait, Iterable,
    QueryFilter, QueryOrder, QuerySelect,
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
            notion_parent_id: value.notion_parent_id,
            parent_type: ParentType::from(value.parent_type),
            created_at: value.created_at.and_utc(),
            updated_at: value.updated_at.map(|f| Utc.from_utc_datetime(&f)),
            contents: value.contents,
            title: value.title,
            draft: value.draft,
        }
    }
}

impl From<PageEntity> for page::ActiveModel {
    fn from(value: PageEntity) -> Self {
        Self {
            notion_page_id: ActiveValue::set(value.notion_page_id),
            notion_parent_id: ActiveValue::set(value.notion_parent_id),
            parent_type: ActiveValue::set(String::from(value.parent_type)),
            created_at: ActiveValue::set(value.created_at.naive_utc()),
            updated_at: ActiveValue::set(
                value.updated_at.map(|f| f.naive_utc()),
            ),
            contents: ActiveValue::set(value.contents),
            title: ActiveValue::set(value.title),
            draft: ActiveValue::set(value.draft),
        }
    }
}

impl PageRepository {
    pub async fn find(
        &self,
        offset: u64,
        limit: u64,
        database_name: Option<String>,
        parent_type: Option<ParentType>,
        has_title: Option<bool>,
    ) -> anyhow::Result<Vec<PageEntity>> {
        let database = NotionDatabase::find()
            .filter(notion_database::Column::Name.eq(database_name))
            .one(&self.db)
            .await?;

        let mut query = Page::find().order_by_desc(page::Column::CreatedAt);

        if let Some(database) = database {
            query = query.filter(page::Column::NotionParentId.eq(database.id));
        }

        if let Some(parent_type) = parent_type {
            query = query
                .filter(page::Column::ParentType.eq(String::from(parent_type)));
        }

        if has_title.is_some() {
            query = query.filter(page::Column::Title.is_not_null());
        }

        let pages = query.limit(limit).offset(offset).all(&self.db).await?;

        Ok(pages.into_iter().map(PageEntity::from).collect())
    }

    pub async fn find_all(&self) -> anyhow::Result<Vec<PageEntity>> {
        let pages = Page::find()
            .order_by_desc(page::Column::CreatedAt)
            .all(&self.db)
            .await?;

        Ok(pages.into_iter().map(PageEntity::from).collect())
    }

    pub async fn find_by_id(
        &self,
        id: &str,
    ) -> anyhow::Result<Option<PageEntity>> {
        let page = page::Entity::find()
            .filter(Column::NotionPageId.eq(id))
            .one(&self.db)
            .await?;

        Ok(page.map(PageEntity::from))
    }

    pub async fn find_by_word(
        &self,
        word: &str,
    ) -> anyhow::Result<Option<PageEntity>> {
        let page = page::Entity::find()
            .filter(Column::Title.contains(word.to_lowercase()))
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

    pub async fn delete(&self, page_id: &str) -> anyhow::Result<()> {
        page::Entity::delete(page::ActiveModel {
            notion_page_id: ActiveValue::Set(page_id.to_string()),
            ..Default::default()
        })
        .exec(&self.db)
        .await?;

        Ok(())
    }
}
