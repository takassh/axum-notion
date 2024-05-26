use sea_orm_migration::prelude::*;

use crate::{
    m20240325_032727_create_page_table::Page,
    m20240517_162442_create_prompt_page_table::PromptPage,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_page_id")
                    .table(PromptPage::Table)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_foreign_key(
                ForeignKeyCreateStatement::new()
                    .name("fk_page_id")
                    .from(PromptPage::Table, PromptPage::PageId)
                    .to(Page::Table, Page::NotionPageId)
                    .to_owned(),
            )
            .await
    }
}
