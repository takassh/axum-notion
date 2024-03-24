use sea_orm_migration::{
    prelude::*,
    sea_orm::{DbBackend, Schema},
};

use crate::entities::event::Entity;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = DbBackend::Postgres;
        let schema = Schema::new(db);
        manager
            .create_table(schema.create_table_from_entity(Entity))
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Entity).to_owned())
            .await
    }
}
