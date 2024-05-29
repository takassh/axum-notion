use sea_orm_migration::prelude::*;

use crate::m20240529_132201_create_nudge_table::Nudge;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Nudge::Table)
                    .add_column(
                        ColumnDef::new(Alias::new("page_id"))
                            .string()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Nudge::Table)
                    .drop_column(Alias::new("page_id"))
                    .to_owned(),
            )
            .await
    }
}
