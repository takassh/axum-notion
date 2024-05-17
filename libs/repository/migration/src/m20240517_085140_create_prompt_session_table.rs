use sea_orm_migration::prelude::*;

use crate::m20240517_085139_create_user_table::User;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PromptSession::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PromptSession::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(PromptSession::UserId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PromptSession::CreatedAt)
                            .date_time()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PromptSession::UpdatedAt)
                            .date_time()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKeyCreateStatement::new()
                            .name("fk_user_id")
                            .from(PromptSession::Table, PromptSession::UserId)
                            .to(User::Table, User::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PromptSession::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum PromptSession {
    Table,
    Id,
    UserId,
    CreatedAt,
    UpdatedAt,
}
