use sea_orm_migration::prelude::*;

use crate::{
    m20240517_085139_create_user_table::User,
    m20240517_085140_create_prompt_session_table::PromptSession,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_user_id")
                    .table(PromptSession::Table)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_foreign_key(
                ForeignKeyCreateStatement::new()
                    .name("fk_user_id")
                    .from(PromptSession::Table, PromptSession::UserId)
                    .to(User::Table, User::Id)
                    .to_owned(),
            )
            .await
    }
}
