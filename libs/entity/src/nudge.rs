use chrono::NaiveDateTime;

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Nudge {
    pub id: i32,
    pub content: String,
    pub page_id: String,
    pub created_at: NaiveDateTime,
}
