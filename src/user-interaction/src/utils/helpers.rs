use crate::errors;
use crate::models::reply::{PostInteractionResponse, ReplyResponse};
use std::collections::HashMap;
use std::fs;
use ulid::Ulid;

pub trait Validate {
    fn validate(&self) -> Result<(), errors::ValidationError>;
}

pub fn get_container_id() -> Option<String> {
    if let Ok(cgroup) = fs::read_to_string("/proc/self/cgroup") {
        for line in cgroup.lines() {
            if let Some(id) = line.split('/').last() {
                if id.len() >= 12 {
                    return Some(id.to_string());
                }
            }
        }
    }
    None
}

pub struct PostInteractionResponseBuilder {
    post_id: Ulid,
    replies: Vec<ReplyResponse>,
    likes: u64,
    views: u64,
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

    pub fn likes(mut self, likes: u64) -> Self {
        self.likes = likes;
        self
    }

    pub fn views(mut self, views: u64) -> Self {
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

fn find_deepest(
    reply_map: &HashMap<Ulid, ReplyResponse>,
    mut parent_reply: ReplyResponse,
) -> ReplyResponse {
    let nested_replies: Vec<ReplyResponse> = reply_map
        .values()
        .filter(|reply| {
            reply.post_id == parent_reply.post_id && reply.reply_to_id == parent_reply.id
        })
        .map(|reply| find_deepest(reply_map, reply.clone()))
        .collect();

    parent_reply.nested_replies = nested_replies;
    parent_reply
}

pub fn map_nested(replies: Vec<ReplyResponse>) -> HashMap<Ulid, Vec<ReplyResponse>> {
    let mut post_map: HashMap<Ulid, Vec<ReplyResponse>> = HashMap::new();

    for reply in replies {
        post_map.entry(reply.post_id).or_default().push(reply);
    }

    post_map
        .into_iter()
        .map(|(post_id, replies)| {
            let mut mapped = Vec::new();
            let reply_map: HashMap<Ulid, ReplyResponse> =
                replies.into_iter().map(|r| (r.id, r)).collect();

            for reply in reply_map.values() {
                if !reply_map.contains_key(&reply.reply_to_id) {
                    let nested = find_deepest(&reply_map, reply.clone());
                    mapped.push(nested);
                }
            }

            (post_id, mapped)
        })
        .collect()
}