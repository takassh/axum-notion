use chrono::{TimeZone, Utc};
use sea_orm::{
    sea_query, ActiveValue, DatabaseConnection, EntityTrait, Iterable,
    QueryFilter,
};

use sea_orm::ColumnTrait;

use crate::active_models::{prelude::*, *};
use entity::prelude::*;

use self::static_page::Column;

#[derive(Clone, Debug)]
pub struct StaticPageRepository {
    db: DatabaseConnection,
}

impl StaticPageRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl From<static_page::Model> for StaticPageEntity {
    fn from(value: static_page::Model) -> Self {
        Self {
            notion_page_id: value.notion_page_id,
            created_at: value.created_at.and_utc(),
            updated_at: value.updated_at.map(|f| Utc.from_utc_datetime(&f)),
        }
    }
}

impl From<StaticPageEntity> for static_page::ActiveModel {
    fn from(value: StaticPageEntity) -> Self {
        Self {
            notion_page_id: ActiveValue::set(value.notion_page_id),
            created_at: ActiveValue::set(value.created_at.naive_utc()),
            updated_at: ActiveValue::set(
                value.updated_at.map(|f| f.naive_utc()),
            ),
        }
    }
}

impl StaticPageRepository {
    pub async fn find_all(&self) -> anyhow::Result<Vec<StaticPageEntity>> {
        let static_pages = StaticPage::find().all(&self.db).await?;

        Ok(static_pages
            .into_iter()
            .map(StaticPageEntity::from)
            .collect())
    }

    pub async fn find_by_id(
        &self,
        id: String,
    ) -> anyhow::Result<Option<StaticPageEntity>> {
        let static_page = static_page::Entity::find()
            .filter(Column::NotionPageId.eq(id))
            .one(&self.db)
            .await?;

        Ok(static_page.map(StaticPageEntity::from))
    }

    pub async fn save(
        &self,
        static_page: StaticPageEntity,
    ) -> anyhow::Result<()> {
        let mut static_page = static_page::ActiveModel::from(static_page);
        static_page.updated_at = ActiveValue::set(Some(Utc::now().naive_utc()));

        let _ = static_page::Entity::insert(static_page)
            .on_conflict(
                sea_query::OnConflict::column(
                    static_page::Column::NotionPageId,
                )
                .update_columns(static_page::Column::iter())
                .to_owned(),
            )
            .exec(&self.db)
            .await?;

        Ok(())
    }

    pub async fn delete(&self, static_page_id: &str) -> anyhow::Result<()> {
        static_page::Entity::delete(static_page::ActiveModel {
            notion_page_id: ActiveValue::Set(static_page_id.to_string()),
            ..Default::default()
        })
        .exec(&self.db)
        .await?;

        Ok(())
    }
}
