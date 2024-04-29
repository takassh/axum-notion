use std::time::Duration;

use block::BlockRepository;
use event::EventRepository;
use migration::Migrator;
use migration::MigratorTrait;
use notion_database::NotionDatabaseRepository;
use page::PageRepository;
use post::PostRepository;
use sea_orm::{ConnectOptions, Database};
use shuttle_persist::PersistInstance;
use top::TopRepository;

mod active_models;
pub mod block;
pub mod event;
pub mod notion_database;
pub mod page;
pub mod post;
pub mod top;

#[derive(Clone, Debug)]
pub struct Repository {
    pub post: PostRepository,
    pub page: PageRepository,
    pub block: BlockRepository,
    pub event: EventRepository,
    pub notion_database_id: NotionDatabaseRepository,
    pub top: Option<TopRepository>,
}

impl Repository {
    pub async fn new(db_url: &str) -> anyhow::Result<Self> {
        let mut opt = ConnectOptions::new(db_url);
        opt.max_connections(5)
            .min_connections(1)
            .sqlx_logging(true)
            .connect_timeout(Duration::from_millis(1000))
            .sqlx_logging_level(log::LevelFilter::Debug);

        let db = Database::connect(opt).await?;

        Migrator::up(&db, None).await?;

        Ok(Self {
            post: PostRepository::new(db.clone()),
            page: PageRepository::new(db.clone()),
            block: BlockRepository::new(db.clone()),
            event: EventRepository::new(db.clone()),
            notion_database_id: NotionDatabaseRepository::new(db.clone()),
            top: None,
        })
    }

    pub fn with_cache(self, cache: PersistInstance) -> Self {
        Self {
            top: Some(TopRepository::new(cache)),
            ..self
        }
    }
}
