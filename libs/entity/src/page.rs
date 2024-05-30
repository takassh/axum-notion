use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct Page {
    pub notion_page_id: String,
    pub notion_parent_id: String,
    pub parent_type: ParentType,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub contents: String,
    pub title: String,
    pub draft: bool,
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub enum ParentType {
    #[default]
    Database,
    Page,
}

impl From<ParentType> for String {
    fn from(value: ParentType) -> Self {
        match value {
            ParentType::Database => "database".to_string(),
            ParentType::Page => "page".to_string(),
        }
    }
}

impl From<String> for ParentType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "database" => ParentType::Database,
            "page" => ParentType::Page,
            _ => ParentType::Database,
        }
    }
}
