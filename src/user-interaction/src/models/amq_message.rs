use chrono::{NaiveDateTime};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use crate::utils::request::ReplyResponse;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PostDeletedMessage {
    #[serde(rename = "postId")]
    pub post_id: Ulid,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PostCreatedMessage {
    #[serde(rename = "postId")]
    pub post_id: Ulid,
}

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
            user_id: value.user.id,
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
            updated_at: updated_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReplyDeletedMessage {
    pub id: Ulid,
}

impl ReplyDeletedMessage {
    pub fn new(id: Ulid) -> Self {
        Self {
            id,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UserDeletedMessage {
    pub id: Ulid,
}

#[derive(Debug, Deserialize)]
pub struct UserCreatedMessage {
    pub id: Ulid,
    pub name: String,
    pub username: String,
}

#[derive(Debug, Deserialize)]
pub struct UserUpdatedMessage {
    pub id: Ulid,
    pub name: String,
    pub bio: String,
    pub location: String,
}