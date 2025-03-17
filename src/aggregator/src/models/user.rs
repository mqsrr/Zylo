use std::ops::Not;
use crate::models::post::{FileMetadata, PaginatedResponse, Post, UserSummary};
use crate::services::aggregator::{FollowRequest, FriendRequests, GrpcUserResponse, RelationshipData};
use chrono::{TimeZone, Utc};
use prost_types::Timestamp;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct User {
    id: String,
    name: String,
    username: String,
    profile_image: FileMetadata,
    background_image: FileMetadata,
    bio: String,
    location: String,
    birthdate: String,
    posts: PaginatedResponse<Post>,
    relationships: UserRelationships,
}

impl User {
    pub fn from(user_response: GrpcUserResponse, posts: PaginatedResponse<Post>, relationships: UserRelationships) -> Self {
        Self {
            id: user_response.id,
            name: user_response.name,
            username: user_response.username,
            profile_image: FileMetadata::from(user_response.profile_image.unwrap()),
            background_image: FileMetadata::from(user_response.background_image.unwrap()),
            bio: user_response.bio.unwrap_or_default(),
            location: user_response.location.unwrap_or_default(),
            birthdate: user_response.birthdate,
            posts,
            relationships,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct UserRelationships {
    friends: UserRelationshipData,
    friend_requests: UserFriendRequests,
    blocks: UserRelationshipData,
    follows: UserFollowRequests,
    #[serde(skip_serializing_if = "<&bool>::not")]
    pub is_stale: bool
}

impl UserRelationships {
    pub fn from(friends: UserRelationshipData, friend_requests: UserFriendRequests, blocks: UserRelationshipData, follows: UserFollowRequests, is_stale: bool) -> Self {
        Self {
            friends,
            friend_requests,
            blocks ,
            follows,
            is_stale
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct UserRelationshipData {
    users: Vec<Arc<UserSummary>>,
    created_at: HashMap<String, String>,
}

impl UserRelationshipData {
    pub fn from(value: RelationshipData, user_map: &HashMap<String, Arc<UserSummary>>) -> Self {
        Self {
            users: value.ids.iter().map(|id| user_map.get(id).unwrap().clone()).collect::<Vec<_>>(),
            created_at: value.created_at.into_iter().filter_map(|(id, timestamp)| {
                google_timestamp_to_string(timestamp).map(|dt_string| (id, dt_string))
            })
                .collect(),
        }
    }
}

fn google_timestamp_to_string(timestamp: Timestamp) -> Option<String> {
    Utc.timestamp_opt(timestamp.seconds, timestamp.nanos as u32)
        .single()
        .map(|dt| dt.to_rfc3339())
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct UserFriendRequests {
    sent: UserRelationshipData,
    received: UserRelationshipData,
}

impl UserFriendRequests {
    pub fn from(value: FriendRequests, user_map: &HashMap<String, Arc<UserSummary>>) -> Self {
        Self {
            sent: UserRelationshipData::from(value.sent.unwrap(), user_map),
            received: UserRelationshipData::from(value.received.unwrap(), user_map),
        }
    }
}

#[derive(Serialize,Deserialize, Debug, Default)]
pub struct UserFollowRequests {
    followers: UserRelationshipData,
    following: UserRelationshipData,
}

impl UserFollowRequests {
    pub fn from(value: FollowRequest, user_map: &HashMap<String, Arc<UserSummary>>) -> Self {
        Self {
            followers: UserRelationshipData::from(value.followers.unwrap(), user_map),
            following: UserRelationshipData::from(value.following.unwrap(), user_map),
        }
    }
}
