use sea_orm_migration::{
    prelude::*,
    sea_orm::{EnumIter, Iterable},
    sea_query::extension::postgres::Type,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(Alias::new("category"))
                    .values(Category::iter())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Post::Table)
                    .if_not_exists()
                    .primary_key(
                        Index::create().col(Post::Id).col(Post::Category),
                    )
                    .col(ColumnDef::new(Post::Id).string().not_null())
                    .col(
                        ColumnDef::new(Post::Category)
                            .enumeration(
                                Alias::new("category"),
                                Category::iter(),
                            )
                            .not_null(),
                    )
                    .col(ColumnDef::new(Post::CreatedAt).date_time().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Post::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(Alias::new("category")).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Post {
    Table,
    Id,
    Category,
    CreatedAt,
}

#[derive(Iden, EnumIter)]
pub enum Category {
    #[iden = "Event"]
    Event,
    #[iden = "Page"]
    Page,
}
