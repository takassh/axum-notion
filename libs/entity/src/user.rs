use chrono::NaiveDateTime;

#[derive(Debug, Default, PartialEq, Clone)]
pub struct User {
    pub id: i32,
    pub sub: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
