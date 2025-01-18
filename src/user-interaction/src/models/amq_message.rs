use crate::utils::request::ReplyResponse;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

pub fn format_datetime(naive: NaiveDateTime) -> String {
    naive.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IdOnlyMessage {
    pub id: Ulid,
}

impl From<Ulid> for IdOnlyMessage {
    fn from(value: Ulid) -> Self {
        Self { id: value }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct PostCreatedMessage {
    pub id: Ulid,
    #[serde(rename = "userId")]
    pub user_id: Ulid,
}

pub type PostDeletedMessage = IdOnlyMessage;
pub type UserCreatedMessage = IdOnlyMessage;
pub type UserDeletedMessage = IdOnlyMessage;
pub type ReplyDeletedMessage = IdOnlyMessage;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReplyCreatedMessage {
    pub id: Ulid,
    #[serde(rename = "userId")]
    pub user_id: Ulid,
    #[serde(rename = "replyToId")]
    pub reply_to_id: Ulid,
    pub content: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

impl From<ReplyResponse> for ReplyCreatedMessage {
    fn from(value: ReplyResponse) -> Self {
        Self {
            id: value.id,
            user_id: value.user_id,
            reply_to_id: value.reply_to_id,
            content: value.content,
            created_at: value.created_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReplyUpdatedMessage {
    pub id: Ulid,
    pub content: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

impl ReplyUpdatedMessage {
    pub fn new(id: Ulid, content: String, updated_at: NaiveDateTime) -> Self {
        Self {
            id,
            content,
            updated_at: format_datetime(updated_at),
        }
    }
}
