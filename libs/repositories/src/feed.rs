use sea_orm::{ConnectionTrait, Statement};
use sea_orm::{DatabaseConnection, EntityTrait};

use crate::{IntoResponse, Response};
use entities::page::{self};

#[derive(Clone, Debug)]
pub struct FeedRepository {
    db: DatabaseConnection,
}

impl FeedRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl FeedRepository {
    pub async fn find_all(&self) -> Response<Vec<page::Model>> {
        Statement::from_sql_and_values(self.db.get_database_backend(), "", []);

        page::Entity::find().all(&self.db).await.into_response("find all")
    }
}
