use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(NotionDatabase::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(NotionDatabase::Id)
                            .primary_key()
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(NotionDatabase::Name)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(NotionDatabase::CreatedAt)
                            .date_time()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(NotionDatabase::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum NotionDatabase {
    Table,
    Id,
    Name,
    CreatedAt,
}
