use sea_orm_migration::prelude::*;

use crate::m20240325_032732_create_post_table::Post;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_index(
                Index::create()
                    .table(Post::Table)
                    .name("idx_created_at")
                    .col(Post::CreatedAt)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .table(Post::Table)
                    .name("idx_created_at")
                    .to_owned(),
            )
            .await
    }
}
