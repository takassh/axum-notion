use sea_orm_migration::prelude::*;

use crate::m20240517_085142_create_prompt_table::Prompt;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Prompt::Table)
                    .add_column(
                        ColumnDef::new(Alias::new("tools_prompt"))
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
                    .table(Prompt::Table)
                    .drop_column(Alias::new("tools_prompt"))
                    .to_owned(),
            )
            .await
    }
}
