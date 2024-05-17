use chrono::NaiveDateTime;

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Prompt {
    pub id: i32,
    pub prompt_session_id: String,
    pub user_prompt: String,
    pub assistant_prompt: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
