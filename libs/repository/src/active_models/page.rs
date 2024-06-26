//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.15

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "page")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub notion_page_id: String,
    pub notion_parent_id: String,
    pub created_at: DateTime,
    pub updated_at: Option<DateTime>,
    pub contents: String,
    pub parent_type: String,
    pub title: String,
    pub draft: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
