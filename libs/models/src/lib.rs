use migration::{Migrator, MigratorTrait};
use repositories::{BlockRepository, PageRepository};
use sea_orm::{ConnectOptions, Database, DbErr};

pub mod entities;
pub mod repositories;
pub use repositories::Repository;

#[derive(Debug, thiserror::Error)]
pub enum ModelsError {
    #[error("Failed to connect db: {}", source)]
    FailedToConnectDB { source: DbErr },

    #[error("Failed to up DB: {}", source)]
    FailedToUpDB { source: DbErr },

    #[error("Repository error: {}", source)]
    RepositoryError { source: DbErr },
}

pub async fn init_repository(db_url: &str) -> Result<Repository, ModelsError> {
    let mut opt = ConnectOptions::new(db_url);
    opt.max_connections(5)
        .min_connections(1)
        .sqlx_logging(true);

    let db = Database::connect(opt)
        .await
        .map_err(|e| ModelsError::FailedToConnectDB { source: e })?;

    Migrator::up(&db, None)
        .await
        .map_err(|e| ModelsError::FailedToUpDB { source: e })?;

    let repository = Repository::new(
        PageRepository::new(db.clone()),
        BlockRepository::new(db.clone()),
    );

    Ok(repository)
}
