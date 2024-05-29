use std::time::Duration;

use block::BlockRepository;
use event::EventRepository;
use migration::Migrator;
use migration::MigratorTrait;
use notion_database::NotionDatabaseRepository;
use nudge::NudgeRepository;
use page::PageRepository;
use post::PostRepository;
use prompt::PromptRepository;
use prompt_session::PromptSessionRepository;
use sea_orm::{ConnectOptions, Database};
use session::SessionRepository;
use shuttle_persist::PersistInstance;
use static_page::StaticPageRepository;
use top::TopRepository;
use user::UserRepository;

mod active_models;
pub mod block;
pub mod event;
pub mod notion_database;
pub mod nudge;
pub mod page;
pub mod post;
pub mod prompt;
pub mod prompt_session;
pub mod session;
pub mod static_page;
pub mod top;
pub mod user;

#[derive(Clone, Debug)]
pub struct Repository {
    pub post: PostRepository,
    pub page: PageRepository,
    pub block: BlockRepository,
    pub event: EventRepository,
    pub user: UserRepository,
    pub prompt_session: PromptSessionRepository,
    pub prompt: PromptRepository,
    pub notion_database_id: NotionDatabaseRepository,
    pub static_page: StaticPageRepository,
    pub nudge: NudgeRepository,
    pub top: Option<TopRepository>,
    pub session: Option<SessionRepository>,
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
            static_page: StaticPageRepository::new(db.clone()),
            block: BlockRepository::new(db.clone()),
            event: EventRepository::new(db.clone()),
            notion_database_id: NotionDatabaseRepository::new(db.clone()),
            user: UserRepository::new(db.clone()),
            prompt_session: PromptSessionRepository::new(db.clone()),
            prompt: PromptRepository::new(db.clone()),
            nudge: NudgeRepository::new(db.clone()),
            top: None,
            session: None,
        })
    }

    pub fn with_cache(self, cache: PersistInstance) -> Self {
        Self {
            top: Some(TopRepository::new(cache)),
            ..self
        }
    }

    pub fn with_session(self, redis: redis::Client) -> Self {
        Self {
            session: Some(SessionRepository::new(redis)),
            ..self
        }
    }
}
