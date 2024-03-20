use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Default)]
#[sea_orm(table_name = "block")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub notion_page_id: String,
    pub updated_at: Option<DateTime>,
    pub contents: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
