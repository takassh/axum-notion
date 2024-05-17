use sea_orm_migration::prelude::*;

use crate::m20240517_085140_create_prompt_session_table::PromptSession;
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Prompt::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Prompt::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Prompt::PromptSessionId)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Prompt::UserPrompt).string().not_null())
                    .col(
                        ColumnDef::new(Prompt::AssistantPrompt)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Prompt::CreatedAt)
                            .date_time()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Prompt::UpdatedAt)
                            .date_time()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKeyCreateStatement::new()
                            .name("fk_prompt_session_id")
                            .from(Prompt::Table, Prompt::PromptSessionId)
                            .to(PromptSession::Table, PromptSession::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Prompt::Table).to_owned())
            .await
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(DeriveIden)]
pub enum Prompt {
    Table,
    Id,
    PromptSessionId,
    UserPrompt,
    AssistantPrompt,
    CreatedAt,
    UpdatedAt,
}
