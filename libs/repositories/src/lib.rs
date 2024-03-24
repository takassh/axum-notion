use block::BlockRepository;
use entities::EntitiesError;
use event::EventRepository;
use page::PageRepository;
use sea_orm::prelude::DbErr;

pub mod block;
pub mod event;
pub mod page;

#[derive(Clone, Debug)]
pub struct Repository {
    pub page: PageRepository,
    pub block: BlockRepository,
    pub event: EventRepository,
}

#[derive(Debug, thiserror::Error)]
pub enum RepositoriesError {
    #[error("Failed to init db: {}", source)]
    FailedToInitDB { source: EntitiesError },

    #[error("Failed to query: {}", source)]
    FailedToQuery { source: DbErr },

    #[error("Failed to save: {}", source)]
    FailedToSave { source: DbErr },
}

pub async fn init_repository(
    db_url: &str,
) -> Result<Repository, RepositoriesError> {
    let db = entities::init_db(db_url)
        .await
        .map_err(|e| RepositoriesError::FailedToInitDB { source: e })?;

    let repository = Repository {
        page: PageRepository::new(db.clone()),
        block: BlockRepository::new(db.clone()),
        event: EventRepository::new(db.clone()),
    };

    Ok(repository)
}
