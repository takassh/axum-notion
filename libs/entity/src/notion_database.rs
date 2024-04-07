use chrono::{DateTime, Utc};

#[derive(Debug, Default, PartialEq, Clone)]
pub struct NotionDatabase {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
}
