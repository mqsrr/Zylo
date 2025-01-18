use std::collections::HashMap;
use chrono::{TimeZone, Utc};
use crate::models::reply::Reply;
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use crate::errors;
use crate::services::grpc_server::reply_server::{FetchPostInteractionsResponse, GrpcPostInteraction, GrpcReply};

pub trait Validate {
    fn validate(&self) -> Result<(), errors::AppError>;
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PostInteractionResponse {
    #[serde(rename="postId")]
    pub post_id: Ulid,
    pub replies: Vec<ReplyResponse>,
    pub likes: i32,
    pub views: i32,
    #[serde(rename="userInteracted", skip_serializing_if = "Option::is_none")]
    pub user_interacted: Option<bool>,
}

pub struct PostInteractionResponseBuilder {
    post_id: Ulid,
    replies: Vec<ReplyResponse>,
    likes: i32,
    views: i32,
    user_interacted: Option<bool>,
}

impl PostInteractionResponseBuilder {
    pub fn new() -> Self {
        Self {
            post_id: Ulid::nil(),
            replies: Vec::new(),
            likes: 0,
            views: 0,
            user_interacted: None,
        }
    }

    pub fn post_id(mut self, post_id: Ulid) -> Self {
        self.post_id = post_id;
        self
    }

    pub fn replies(mut self, replies: Vec<ReplyResponse>) -> Self {
        self.replies = replies;
        self
    }

    pub fn likes(mut self, likes: i32) -> Self {
        self.likes = likes;
        self
    }

    pub fn views(mut self, views: i32) -> Self {
        self.views = views;
        self
    }

    pub fn user_interacted(mut self, user_interacted: Option<bool>) -> Self {
        self.user_interacted = user_interacted;
        self
    }

    pub fn build(self) -> PostInteractionResponse {
        PostInteractionResponse {
            post_id: self.post_id,
            replies: self.replies,
            likes: self.likes,
            views: self.views,
            user_interacted: self.user_interacted,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct CreateReplyRequest {
    #[serde(rename="userId")]
    pub user_id: Ulid,
    #[serde(rename="replyToId")]
    pub reply_to_id: Ulid,
    pub content: String,
}

impl  Reply {
    pub fn from_create(value: CreateReplyRequest, id: Ulid, root_id: Ulid, path: String) -> Self {
        Self {
            id,
            user_id: value.user_id,
            reply_to_id: value.reply_to_id,
            content: value.content,
            created_at: Utc::now().naive_utc(),
            root_id,
            path,
        }
    }
}

impl Validate for CreateReplyRequest {
    fn validate(&self) -> Result<(), errors::AppError> {
        if self.content.is_empty() {  
            return Err(errors::AppError::ValidationError("content can not be empty".to_string()))
        }
        
        Ok(())
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct UpdateReplyRequest {
    pub content: String,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReplyResponse {
    pub id: Ulid,
    pub user_id: Ulid,
    #[serde(rename="replyToId")]
    pub reply_to_id: Ulid,
    pub content: String,
    #[serde(rename="createdAt")]
    pub created_at: String,
    #[serde(rename="nestedReplies")]
    pub nested_replies: Vec<ReplyResponse>,
    pub likes: i32,
    pub views: i32,
    #[serde(rename="userInteracted", skip_serializing_if = "Option::is_none")]
    pub user_interacted: Option<bool>,
}

impl From<Reply> for ReplyResponse {
    fn from(value: Reply) -> Self {
        Self {
            id: value.id,
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

impl From<ReplyResponse> for GrpcReply {
    fn from(reply_response: ReplyResponse) -> Self {
        Self {
            id: reply_response.id.to_string(),
            content: reply_response.content,
            user_id: reply_response.user_id.to_string(),
            reply_to_id: reply_response.reply_to_id.to_string(),
            created_at: reply_response
                .created_at
                .parse::<i64>()
                .unwrap_or(Utc::now().timestamp()),
            nested_replies: reply_response
                .nested_replies
                .into_iter()
                .map(GrpcReply::from)
                .collect(),
            likes: reply_response.likes,
            views: reply_response.views,
            user_interacted: reply_response.user_interacted.unwrap_or(false),
        }
    }
}



impl From<PostInteractionResponse> for FetchPostInteractionsResponse {
    fn from(response: PostInteractionResponse) -> Self {
        Self {
            post_interaction: Some(GrpcPostInteraction {
                post_id: response.post_id.to_string(),
                replies: response.replies.into_iter().map(GrpcReply::from).collect(),
                likes: response.likes,
                views: response.views,
                user_interacted: response.user_interacted.unwrap_or(false),
            }),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PostDeletedMessage {
    pub post_id: Ulid
}

fn find_deepest(replies: &Vec<ReplyResponse>, parent_reply: ReplyResponse) -> ReplyResponse {
    let mut reply_with_nested = parent_reply.clone();
    let nested_replies: Vec<ReplyResponse> = replies
        .iter()
        .filter(|reply| reply.reply_to_id == reply_with_nested.id)
        .map(|reply| find_deepest(replies, reply.clone()))
        .collect();

    reply_with_nested.nested_replies = nested_replies;
    reply_with_nested
}

pub fn map_nested(replies: Vec<ReplyResponse>) -> Vec<ReplyResponse> {
    let mut mapped_replies = Vec::new();
    let reply_map: HashMap<Ulid, ReplyResponse> = replies.into_iter().map(|r| (r.id, r)).collect();

    for reply in reply_map.values() {
        if reply_map.contains_key(&reply.reply_to_id) {
            continue;
        }

        let nested_reply = find_deepest(&reply_map.values().cloned().collect(), reply.clone());
        mapped_replies.push(nested_reply);
    }

    mapped_replies
}