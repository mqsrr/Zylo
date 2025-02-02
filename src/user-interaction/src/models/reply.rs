use chrono::{NaiveDateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use crate::errors;
use crate::utils::helpers::Validate;

#[derive(Debug, Clone)]
pub struct Reply {
    pub id: Ulid,
    pub post_id: Ulid,
    pub reply_to_id: Ulid,
    pub user_id: Ulid,
    pub content: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReplyResponse {
    pub id: Ulid,
    #[serde(skip_serializing)]
    pub post_id: Ulid,
    pub user_id: Ulid,
    #[serde(rename="replyToId")]
    pub reply_to_id: Ulid,
    pub content: String,
    #[serde(rename="createdAt")]
    pub created_at: String,
    #[serde(rename="nestedReplies")]
    pub nested_replies: Vec<ReplyResponse>,
    pub likes: u64,
    pub views: u64,
    #[serde(rename="userInteracted", skip_serializing_if = "Option::is_none")]
    pub user_interacted: Option<bool>,
}

impl From<Reply> for ReplyResponse {
    fn from(value: Reply) -> Self {
        Self {
            id: value.id,
            post_id: value.post_id,
            user_id: value.user_id,
            reply_to_id: value.reply_to_id,
            content: value.content,
            created_at: Utc.from_utc_datetime(&value.created_at).to_rfc3339(),
            nested_replies: Vec::<ReplyResponse>::new(),
            likes: 0,
            views: 0,
            user_interacted: None,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct UpdateReplyRequest {
    pub content: String,
}


#[derive(Debug, Deserialize, Clone)]
pub struct CreateReplyRequest {
    #[serde(rename="userId")]
    pub user_id: Ulid,
    #[serde(rename="replyToId")]
    pub reply_to_id: Ulid,
    pub content: String,
}


impl Validate for CreateReplyRequest {
    fn validate(&self) -> Result<(), errors::ValidationError> {
        if self.content.is_empty() {
            return Err(errors::ValidationError::Failed("content can not be empty".to_string()))
        }
        
        if self.reply_to_id.is_nil() {
            return Err(errors::ValidationError::InvalidReplyToId)
        }   
        
        if self.user_id.is_nil() {
            return Err(errors::ValidationError::InvalidUserId)
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PostInteractionResponse {
    #[serde(rename = "postId")]
    pub post_id: Ulid,
    pub replies: Vec<ReplyResponse>,
    pub likes: u64,
    pub views: u64,
    #[serde(rename = "userInteracted", skip_serializing_if = "Option::is_none")]
    pub user_interacted: Option<bool>,
}