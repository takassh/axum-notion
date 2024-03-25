use block::BlockRepository;
use event::EventRepository;
use migration::Migrator;
use migration::MigratorTrait;
use page::PageRepository;
use post::PostRepository;
use response::IntoResponse;
use response::Response;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};

mod active_models;
pub mod block;
pub mod event;
pub mod page;
pub mod post;
mod response;

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error(
        "in sea-orm crate from unsuccessful database operations: {}: {}",
        message,
        source
    )]
    InSeaOrmDbErr {
        message: String,
        source: sea_orm::DbErr,
    },

    #[error("unimplemented yet")]
    Unimplemented,
}

#[derive(Clone, Debug)]
pub struct Repository {
    pub feed: PostRepository,
    pub page: PageRepository,
    pub block: BlockRepository,
    pub event: EventRepository,
}

pub async fn init_repository(db_url: &str) -> Response<Repository> {
    let db = init_db(db_url).await?;

    let repository = Repository {
        feed: PostRepository::new(db.clone()),
        page: PageRepository::new(db.clone()),
        block: BlockRepository::new(db.clone()),
        event: EventRepository::new(db.clone()),
    };

    Ok(repository)
}

async fn init_db(db_url: &str) -> Response<DatabaseConnection> {
    let mut opt = ConnectOptions::new(db_url);
    opt.max_connections(5)
        .min_connections(1)
        .sqlx_logging(true)
        .sqlx_logging_level(log::LevelFilter::Debug);

    let db = Database::connect(opt)
        .await
        .into_response("in database connect")?;

    Migrator::up(&db, None)
        .await
        .into_response("in migrator up")?;

    Ok(db)
}
