use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct Block {
    pub notion_page_id: String,
    pub updated_at: Option<DateTime<Utc>>,
    pub contents: String,
}
