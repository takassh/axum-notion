pub use sea_orm_migration::prelude::*;

mod m20240219_000001_create_block_table;
mod m20240219_000001_create_page_table;

use sea_orm::prelude::DatabaseConnection;
use sea_orm::{ConnectOptions, Database};

mod entities;
pub use entities::*;

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

#[derive(Debug, thiserror::Error)]
pub enum EntitiesError {
    #[error("Failed to connect db: {}", source)]
    FailedToConnectDB { source: DbErr },

    #[error("Failed to up DB: {}", source)]
    FailedToUpDB { source: DbErr },
}

pub async fn init_db(
    db_url: &str,
) -> Result<DatabaseConnection, EntitiesError> {
    let mut opt = ConnectOptions::new(db_url);
    opt.max_connections(5).min_connections(1).sqlx_logging(true);

    let db = Database::connect(opt)
        .await
        .map_err(|e| EntitiesError::FailedToConnectDB { source: e })?;

    Migrator::up(&db, None)
        .await
        .map_err(|e| EntitiesError::FailedToUpDB { source: e })?;

    Ok(db)
}
