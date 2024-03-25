use chrono::{DateTime, Utc};

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Block {
    pub notion_page_id: String,
    pub updated_at: Option<DateTime<Utc>>,
    pub contents: String,
}
