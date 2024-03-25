use chrono::{DateTime, Utc};

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Event {
    pub github_event_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub contents: String,
}
