use sea_orm_migration::prelude::*;

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
                        ColumnDef::new(Page::ParentType)
                            .string()
                            .not_null()
                            .default("database"),
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
                    .drop_column(Page::ParentType)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Page {
    Table,
    ParentType,
}
