use sea_orm_migration::prelude::*;

use crate::m20240325_032727_create_page_table::Page;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Page::Table)
                    .add_column(
                        ColumnDef::new(Alias::new("title"))
                            .string()
                            .not_null()
                            .default(""),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Page::Table)
                    .drop_column(Alias::new("title"))
                    .to_owned(),
            )
            .await
    }
}
