use chrono::Utc;
use sea_orm::{
    sea_query, DatabaseConnection, EntityTrait, Iterable, QueryFilter,
};

use entities::page::{self, Column};
use sea_orm::ColumnTrait;
use sea_orm::IntoActiveModel;

use crate::IntoResponse;
use crate::Response;

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
    pub async fn find_all(
        &self,
    ) -> Response<Vec<page::Model>> {
        page::Entity::find()
            .all(&self.db)
            .await
            .into_response("find all")
    }

    pub async fn find_by_id(
        &self,
        id: String,
    ) -> Response<Option<page::Model>> {
        page::Entity::find()
            .filter(Column::NotionPageId.eq(id))
            .one(&self.db)
            .await
            .into_response("find by id")
    }

    pub async fn save(
        &self,
        mut page: page::Model,
    ) -> Response<()> {
        page.updated_at = Some(Utc::now().naive_utc());
        let _ = page::Entity::insert(page.into_active_model())
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
