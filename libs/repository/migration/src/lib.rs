pub use sea_orm_migration::prelude::*;

mod m20240325_032714_create_event_table;
mod m20240325_032727_create_page_table;
mod m20240325_032732_create_post_table;
mod m20240325_032828_create_block_table;
mod m20240325_125942_create_index_at_post;

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
        ]
    }
}
