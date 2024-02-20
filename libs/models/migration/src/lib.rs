pub use sea_orm_migration::prelude::*;

mod m20240219_000001_create_block_table;
mod m20240219_000001_create_page_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240219_000001_create_block_table::Migration),
            Box::new(m20240219_000001_create_page_table::Migration),
        ]
    }
}
