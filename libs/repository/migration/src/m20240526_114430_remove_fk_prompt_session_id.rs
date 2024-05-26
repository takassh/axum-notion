use sea_orm_migration::prelude::*;

use crate::{
    m20240517_085140_create_prompt_session_table::PromptSession,
    m20240517_085142_create_prompt_table::Prompt,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_prompt_session_id")
                    .table(Prompt::Table)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_foreign_key(
                ForeignKeyCreateStatement::new()
                    .name("fk_prompt_session_id")
                    .from(Prompt::Table, Prompt::PromptSessionId)
                    .to(PromptSession::Table, PromptSession::Id)
                    .to_owned(),
            )
            .await
    }
}
