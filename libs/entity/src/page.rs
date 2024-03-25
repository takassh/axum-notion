use chrono::{DateTime, Utc};

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Page {
    pub notion_page_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub contents: String,
}
