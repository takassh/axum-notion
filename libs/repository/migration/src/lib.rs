pub use sea_orm_migration::prelude::*;

mod m20240325_032714_create_event_table;
mod m20240325_032727_create_page_table;
mod m20240325_032732_create_post_table;
mod m20240325_032828_create_block_table;
mod m20240325_125942_create_index_at_post;
mod m20240407_034855_create_notion_database_table;
mod m20240512_100556_create_static_page_table;
mod m20240512_104914_rename_to_notion_parent_id_table;
mod m20240512_110241_add_parent_type_column;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240325_032714_create_event_table::Migration),
            Box::new(m20240325_032727_create_page_table::Migration),
            Box::new(m20240325_032732_create_post_table::Migration),
            Box::new(m20240325_032828_create_block_table::Migration),
            Box::new(m20240325_125942_create_index_at_post::Migration),
            Box::new(m20240407_034855_create_notion_database_table::Migration),
            Box::new(m20240512_100556_create_static_page_table::Migration),
            Box::new(
                m20240512_104914_rename_to_notion_parent_id_table::Migration,
            ),
            Box::new(m20240512_110241_add_parent_type_column::Migration),
        ]
    }
}
