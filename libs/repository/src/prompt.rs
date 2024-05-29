use chrono::{NaiveDateTime, Utc};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, EntityTrait};

use crate::active_models::{prelude::*, *};
use entity::prelude::*;

#[derive(Clone, Debug)]
pub struct PromptRepository {
    db: DatabaseConnection,
}

impl PromptRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl From<prompt::Model> for PromptEntity {
    fn from(value: prompt::Model) -> Self {
        PromptEntity {
            id: value.id,
            prompt_session_id: value.prompt_session_id,
            user_prompt: value.user_prompt,
            assistant_prompt: value.assistant_prompt,
            tools_prompt: value.tools_prompt,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl From<PromptEntity> for prompt::ActiveModel {
    fn from(value: PromptEntity) -> Self {
        Self {
            id: if value.id == i32::default() {
                ActiveValue::not_set()
            } else {
                ActiveValue::Set(value.id)
            },
            prompt_session_id: ActiveValue::Set(value.prompt_session_id),
            user_prompt: ActiveValue::Set(value.user_prompt),
            assistant_prompt: ActiveValue::Set(value.assistant_prompt),
            tools_prompt: ActiveValue::Set(value.tools_prompt),
            created_at: if value.created_at == NaiveDateTime::default() {
                ActiveValue::Set(Utc::now().naive_utc())
            } else {
                ActiveValue::Set(value.created_at)
            },
            updated_at: ActiveValue::Set(Utc::now().naive_utc()),
        }
    }
}

impl PromptRepository {
    pub async fn find_all(&self) -> anyhow::Result<Vec<PromptEntity>> {
        let prompts = Prompt::find().all(&self.db).await?;

        Ok(prompts.into_iter().map(PromptEntity::from).collect())
    }

    pub async fn save(
        &self,
        prompt: PromptEntity,
        page_ids: Vec<String>,
    ) -> anyhow::Result<i32> {
        let prompt = prompt::ActiveModel::from(prompt).save(&self.db).await?;
        let prompt_id = prompt.id.unwrap();
        for page_id in page_ids {
            prompt_page::ActiveModel {
                id: ActiveValue::not_set(),
                prompt_id: ActiveValue::Set(prompt_id),
                page_id: ActiveValue::Set(page_id),
                created_at: ActiveValue::Set(Utc::now().naive_utc()),
            }
            .save(&self.db)
            .await?;
        }

        Ok(prompt_id)
    }

    pub async fn delete(&self, prompt_id: i32) -> anyhow::Result<()> {
        prompt::Entity::delete(prompt::ActiveModel {
            id: ActiveValue::Set(prompt_id),
            ..Default::default()
        })
        .exec(&self.db)
        .await?;

        Ok(())
    }
}
