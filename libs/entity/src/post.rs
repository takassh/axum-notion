use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Post {
    pub id: String,
    pub category: Category,
    pub contents: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(
    Debug, Default, PartialEq, Clone, Serialize, Deserialize, strum::EnumIter,
)]
pub enum Category {
    #[default]
    Event,
    Page,
}
