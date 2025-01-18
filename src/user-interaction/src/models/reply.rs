use chrono::NaiveDateTime;
use ulid::Ulid;

#[derive(Debug, Clone)]
pub struct Reply {
    pub id: Ulid,
    pub root_id: Ulid,
    pub reply_to_id: Ulid,
    pub user_id: Ulid,
    pub content: String,
    pub created_at: NaiveDateTime,
    pub path: String,
}