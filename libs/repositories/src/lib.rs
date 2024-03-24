use block::BlockRepository;
use entities::EntitiesError;
use event::EventRepository;
use feed::FeedRepository;
use page::PageRepository;
use sea_orm::prelude::DbErr;

pub mod block;
pub mod event;
pub mod feed;
pub mod page;

#[derive(Clone, Debug)]
pub struct Repository {
    pub feed: FeedRepository,
    pub page: PageRepository,
    pub block: BlockRepository,
    pub event: EventRepository,
}

#[derive(Debug, thiserror::Error)]
pub enum RepositoriesError {
    #[error("entities: {}: {}", message, source)]
    EntitiesError {
        message: String,
        source: EntitiesError,
    },

    #[error("db: {}: {}", message, source)]
    DbErr { message: String, source: DbErr },
}

type Response<T> = Result<T, RepositoriesError>;

pub trait IntoResponse<T> {
    fn into_response(self, message: &str) -> Response<T>;
}

impl<T> IntoResponse<T> for Result<T, DbErr> {
    fn into_response(self, message: &str) -> Response<T> {
        self.map_err(|e| RepositoriesError::DbErr {
            message: message.to_string(),
            source: e,
        })
    }
}

impl<T> IntoResponse<T> for Result<T, EntitiesError> {
    fn into_response(self, message: &str) -> Response<T> {
        self.map_err(|e| RepositoriesError::EntitiesError {
            message: message.to_string(),
            source: e,
        })
    }
}

pub async fn init_repository(db_url: &str) -> Response<Repository> {
    let db = entities::init_db(db_url).await.into_response("failed to init")?;

    let repository = Repository {
        feed: FeedRepository::new(db.clone()),
        page: PageRepository::new(db.clone()),
        block: BlockRepository::new(db.clone()),
        event: EventRepository::new(db.clone()),
    };

    Ok(repository)
}
