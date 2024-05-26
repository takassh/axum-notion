use sea_orm_migration::prelude::*;

use crate::{
    m20240325_032727_create_page_table::Page,
    m20240517_085142_create_prompt_table::Prompt,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PromptPage::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PromptPage::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(PromptPage::PromptId)
                            .integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(PromptPage::PageId).string().not_null())
                    .col(
                        ColumnDef::new(PromptPage::CreatedAt)
                            .date_time()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKeyCreateStatement::new()
                            .name("fk_prompt_id")
                            .from(PromptPage::Table, PromptPage::PromptId)
                            .to(Prompt::Table, Prompt::Id),
                    )
                    .foreign_key(
                        ForeignKeyCreateStatement::new()
                            .name("fk_page_id")
                            .from(PromptPage::Table, PromptPage::PageId)
                            .to(Page::Table, Page::NotionPageId),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PromptPage::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum PromptPage {
    Table,
    Id,
    PromptId,
    PageId,
    CreatedAt,
}
