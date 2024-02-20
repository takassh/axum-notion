use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Block::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Block::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(Block::Contents).string().not_null())
                    .col(ColumnDef::new(Block::UpdatedAt).date_time())
                    .col(ColumnDef::new(Block::CreatedAt).date_time().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Block::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Block {
    Table,
    Id,
    Contents,
    CreatedAt,
    UpdatedAt,
}
