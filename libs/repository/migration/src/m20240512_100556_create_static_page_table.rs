use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(StaticPage::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(StaticPage::NotionPageId)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(StaticPage::CreatedAt)
                            .date_time()
                            .not_null(),
                    )
                    .col(ColumnDef::new(StaticPage::UpdatedAt).date_time())
                    .col(
                        ColumnDef::new(StaticPage::Contents)
                            .string()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(StaticPage::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum StaticPage {
    Table,
    NotionPageId,
    CreatedAt,
    UpdatedAt,
    Contents,
}
