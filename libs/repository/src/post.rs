use sea_orm::{
    sea_query, strum::IntoEnumIterator as _, ActiveValue, DatabaseConnection,
    EntityTrait, QueryFilter,
};
use sea_orm::{ColumnTrait, QueryOrder};
use std::collections::HashMap;

use crate::active_models::{prelude::*, *};
use entity::prelude::*;
use sea_orm::IntoActiveValue;
use strum::IntoEnumIterator as _;

use self::sea_orm_active_enums::Category;

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
            contents: None,
            category: value.category.into(),
            created_at: value.created_at.and_utc(),
        }
    }
}

impl PostRepository {
    pub async fn find_all(&self) -> anyhow::Result<Vec<PostEntity>> {
        let posts = Post::find()
            .order_by_desc(post::Column::CreatedAt)
            .all(&self.db)
            .await?;
        let event_ids: Vec<_> = posts
            .iter()
            .filter(|x| x.category == Category::Event)
            .map(|x| x.id.clone())
            .collect();
        let page_ids: Vec<_> = posts
            .iter()
            .filter(|x| x.category == Category::Page)
            .map(|x| x.id.clone())
            .collect();

        let events: HashMap<_, _> = Event::find()
            .filter(event::Column::GithubEventId.is_in(event_ids))
            .all(&self.db)
            .await?
            .iter()
            .map(|x| (x.github_event_id.clone(), x.contents.clone()))
            .collect();
        let pages: HashMap<_, _> = Page::find()
            .filter(page::Column::NotionPageId.is_in(page_ids))
            .all(&self.db)
            .await?
            .iter()
            .map(|x| (x.notion_page_id.clone(), x.contents.clone()))
            .collect();

        let mut results = vec![];
        for post in posts {
            let contents = match post.category {
                Category::Event => events.get(&post.id).cloned(),
                Category::Page => pages.get(&post.id).cloned(),
            };
            results.push(PostEntity {
                id: post.id,
                contents,
                category: post.category.into(),
                created_at: post.created_at.and_utc(),
            });
        }

        Ok(results)
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
