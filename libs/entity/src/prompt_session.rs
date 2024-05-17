use chrono::NaiveDateTime;

#[derive(Debug, Default, PartialEq, Clone)]
pub struct PromptSession {
    pub id: String,
    pub user_id: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
