//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.15

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "prompt_page")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub prompt_id: i32,
    pub page_id: String,
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::page::Entity",
        from = "Column::PageId",
        to = "super::page::Column::NotionPageId",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Page,
    #[sea_orm(
        belongs_to = "super::prompt::Entity",
        from = "Column::PromptId",
        to = "super::prompt::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Prompt,
}

impl Related<super::page::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Page.def()
    }
}

impl Related<super::prompt::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Prompt.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
