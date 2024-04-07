use block::BlockRepository;
use event::EventRepository;
use migration::Migrator;
use migration::MigratorTrait;
use notion_database::NotionDatabaseRepository;
use page::PageRepository;
use post::PostRepository;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};

mod active_models;
pub mod block;
pub mod event;
pub mod notion_database;
pub mod page;
pub mod post;

#[derive(Clone, Debug)]
pub struct Repository {
    pub post: PostRepository,
    pub page: PageRepository,
    pub block: BlockRepository,
    pub event: EventRepository,
    pub notion_database_id: NotionDatabaseRepository,
}

pub async fn init_repository(db_url: &str) -> anyhow::Result<Repository> {
    let db = init_db(db_url).await?;

    let repository = Repository {
        post: PostRepository::new(db.clone()),
        page: PageRepository::new(db.clone()),
        block: BlockRepository::new(db.clone()),
        event: EventRepository::new(db.clone()),
        notion_database_id: NotionDatabaseRepository::new(db.clone()),
    };

    Ok(repository)
}

async fn init_db(db_url: &str) -> anyhow::Result<DatabaseConnection> {
    let mut opt = ConnectOptions::new(db_url);
    opt.max_connections(5)
        .min_connections(1)
        .sqlx_logging(true)
        .sqlx_logging_level(log::LevelFilter::Debug);

    let db = Database::connect(opt).await?;

    Migrator::up(&db, None).await?;

    Ok(db)
}
