use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Page::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Page::NotionPageId)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Page::CreatedAt).date_time().not_null())
                    .col(ColumnDef::new(Page::UpdatedAt).date_time())
                    .col(ColumnDef::new(Page::Contents).string().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Page::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Page {
    Table,
    NotionPageId,
    CreatedAt,
    UpdatedAt,
    Contents,
}
