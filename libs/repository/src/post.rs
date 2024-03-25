use sea_orm::{
    sea_query, strum::IntoEnumIterator as _, ActiveValue, DatabaseConnection,
    EntityTrait,
};

use crate::active_models::{prelude::*, *};
use entity::prelude::*;
use sea_orm::IntoActiveValue;
use strum::IntoEnumIterator as _;

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
            category: value.category.into(),
            created_at: value.created_at.and_utc(),
        }
    }
}

impl PostRepository {
    pub async fn find_all(&self) -> anyhow::Result<Vec<PostEntity>> {
        let posts = Post::find().all(&self.db).await?;

        Ok(posts.into_iter().map(PostEntity::from).collect())
    }

    pub async fn save(&self, post: PostEntity) -> anyhow::Result<()> {
        let category: sea_orm_active_enums::Category = post.category.into();
        let model = post::ActiveModel {
            id: post.id.into_active_value(),
            category: ActiveValue::set(category),
            created_at: post.created_at.naive_utc().into_active_value(),
        };

        let _ = Post::insert(model)
            .on_conflict(
                sea_query::OnConflict::columns([
                    post::Column::Id,
                    post::Column::Category,
                ])
                .update_columns(post::Column::iter())
                .to_owned(),
            )
            .exec(&self.db)
            .await?;

        Ok(())
    }
}

macro_rules! impl_from {
    ($from:ty, $to:ty) => {
        impl From<$from> for $to {
            fn from(value: $from) -> Self {
                <$to>::iter()
                    .find(|x| (x.clone() as usize) == (value.clone() as usize))
                    .unwrap()
            }
        }

        impl From<$to> for $from {
            fn from(value: $to) -> Self {
                <$from>::iter()
                    .find(|x| (x.clone() as usize) == (value.clone() as usize))
                    .unwrap()
            }
        }
    };
}

impl_from!(entity::post::Category, sea_orm_active_enums::Category);
