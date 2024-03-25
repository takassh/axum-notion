use sea_orm::{DatabaseConnection, EntityTrait};

use crate::active_models::{prelude::*, *};
use entity::prelude::*;

#[derive(Clone, Debug)]
pub struct PostRepository {
    db: DatabaseConnection,
}

impl PostRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl From<post::Model> for PostEntity {
    fn from(value: post::Model) -> Self {
        Self {
            id: value.id,
            category: match value.category {
                sea_orm_active_enums::Category::Feed => {
                    entity::post::Category::Feed
                }
                sea_orm_active_enums::Category::Page => {
                    entity::post::Category::Page
                }
            },
            created_at: value.created_at.and_utc(),
        }
    }
}

impl PostRepository {
    pub async fn find_all(&self) -> anyhow::Result<Vec<PostEntity>> {
        let posts = Post::find().all(&self.db).await?;

        Ok(posts.into_iter().map(PostEntity::from).collect())
    }
}
