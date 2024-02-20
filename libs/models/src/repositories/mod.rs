use sea_orm::DatabaseConnection;

pub mod block;
pub mod page;

#[derive(Clone, Debug)]
pub struct Repository {
    pub page: PageRepository,
    pub block: BlockRepository,
}

impl Repository {
    pub fn new(page: PageRepository, block: BlockRepository) -> Self {
        Self { page, block }
    }
}

#[derive(Clone, Debug)]
pub struct PageRepository {
    db: DatabaseConnection,
}

#[derive(Clone, Debug)]
pub struct BlockRepository {
    db: DatabaseConnection,
}
