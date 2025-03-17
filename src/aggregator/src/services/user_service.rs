use crate::errors;
use crate::models::user::{
    User, UserFollowRequests, UserFriendRequests, UserRelationshipData, UserRelationships,
};
use std::collections::{HashMap, HashSet};

use crate::models::post::{PaginatedResponse, Post};
use crate::services::InjectTraceContext;
use crate::services::aggregator::post_service_client::PostServiceClient;
use crate::services::aggregator::relationship_service_client::RelationshipServiceClient;
use crate::services::aggregator::reply_service_client::ReplyServiceClient;
use crate::services::aggregator::user_profile_service_client::UserProfileServiceClient;
use crate::services::aggregator::{BatchOfPostInteractionsResponse, GetBatchOfPostInteractionsRequest, GetUserByIdRequest,  PostsRequest, RelationshipRequest, RelationshipResponse};
use async_trait::async_trait;
use tonic::IntoRequest;
use tonic::transport::Channel;
use tracing::log::{error};
use ulid::Ulid;
use crate::utils::helpers::{collect_user_ids_from_reply, fetch_user_summaries};

#[async_trait]
pub trait UserService: Send + Sync {
    async fn get_by_id(
        &mut self,
        id: Ulid,
        interaction_user_id: Option<Ulid>,
    ) -> Result<User, errors::GrpcError>;
}

pub struct UserServiceImpl {
    user_client: UserProfileServiceClient<Channel>,
    relationship_client: RelationshipServiceClient<Channel>,
    post_client: PostServiceClient<Channel>,
    reply_client: ReplyServiceClient<Channel>,
}

impl UserServiceImpl {
    pub fn new(
        user_client: UserProfileServiceClient<Channel>,
        relationship_client: RelationshipServiceClient<Channel>,
        post_client: PostServiceClient<Channel>,
        reply_client: ReplyServiceClient<Channel>,
    ) -> Self {
        Self {
            user_client,
            relationship_client,
            post_client,
            reply_client,
        }
    }

}

#[async_trait]
impl UserService for UserServiceImpl {
    async fn get_by_id(
        &mut self,
        id: Ulid,
        interaction_user_id: Option<Ulid>,
    ) -> Result<User, errors::GrpcError> {
        let user_id = id.to_string();
        let request = GetUserByIdRequest { user_id: user_id.clone() }
            .into_request()
            .inject_trace_context();

        let user = self.user_client.get_user_by_id(request).await?.into_inner();

        let request = PostsRequest {
            user_id: Some(user_id.clone()),
            per_page: 10,
            last_post_id: None,
        }
            .into_request()
            .inject_trace_context();

        let paginated_posts = self
            .post_client
            .get_paginated_posts(request)
            .await?
            .into_inner();

        let relationship_request = RelationshipRequest { user_id }
            .into_request()
            .inject_trace_context();

        let mut interactions = BatchOfPostInteractionsResponse::default();
        let mut interactions_stale = false;
        if !paginated_posts.posts.is_empty() {
            let interaction_request = GetBatchOfPostInteractionsRequest {
                posts_ids: paginated_posts.posts.iter().map(|p| p.id.clone()).collect(),
                interaction_user_id: interaction_user_id.unwrap_or(id).to_string(),
            }
                .into_request()
                .inject_trace_context();

            interactions = self
                .reply_client
                .get_batch_of_post_interactions(interaction_request)
                .await
                .map(|i| i.into_inner())
                .unwrap_or_else(|e| {
                    error!("Failed to retrieve post interactions: {:?}", e);
                    interactions_stale = true;
                    BatchOfPostInteractionsResponse::default()
                });
        }

        let mut relationships_stale = false;
        let relationships = self
            .relationship_client
            .get_user_relationships(relationship_request)
            .await
            .map(|res| res.into_inner())
            .unwrap_or_else(|e| {
                error!("Failed to retrieve user relationships: {:?}", e);
                relationships_stale = true;
                RelationshipResponse::default()
            });

        let mut user_ids = HashSet::new();
        for post in &paginated_posts.posts {
            user_ids.insert(post.user_id.clone());
        }
        for interaction in &interactions.posts_interactions {
            for reply in &interaction.replies {
                collect_user_ids_from_reply(reply, &mut user_ids)
            }
        }

        if let Some(ref rel_data) = relationships.relationships {
            if let Some(ref follows) = rel_data.follows {
                if let (Some(followers), Some(following)) =
                    (follows.followers.as_ref(), follows.following.as_ref())
                {
                    user_ids.extend(followers.ids.iter().cloned());
                    user_ids.extend(following.ids.iter().cloned());
                }
            }
            if let Some(ref blocks) = rel_data.blocks {
                user_ids.extend(blocks.ids.iter().cloned());
            }
            if let Some(ref friends) = rel_data.friends {
                user_ids.extend(friends.ids.iter().cloned());
            }
            if let Some(ref friend_requests) = rel_data.friend_requests {
                if let (Some(received), Some(sent)) =
                    (friend_requests.received.as_ref(), friend_requests.sent.as_ref())
                {
                    user_ids.extend(received.ids.iter().cloned());
                    user_ids.extend(sent.ids.iter().cloned());
                }
            }
        }

        let user_map = fetch_user_summaries(&mut self.user_client, user_ids)
            .await
            .unwrap_or_else(|e| {
                error!("Failed to retrieve user data: {:?}", e);
                HashMap::default()
            });

        let mut user_relationships = if let Some(rel_data) = relationships.relationships {
            if let (Some(follows), Some(friend_requests), Some(blocks), Some(friends)) =
                (rel_data.follows, rel_data.friend_requests, rel_data.blocks, rel_data.friends)
            {
                UserRelationships::from(
                    UserRelationshipData::from(friends, &user_map),
                    UserFriendRequests::from(friend_requests, &user_map),
                    UserRelationshipData::from(blocks, &user_map),
                    UserFollowRequests::from(follows, &user_map),
                    relationships_stale
                )
            } else {
                UserRelationships::default()
            }
        } else {
            UserRelationships::default()
        };
        
        user_relationships.is_stale = relationships_stale;
        Ok(User::from(
            user,
            PaginatedResponse::<Post>::from(paginated_posts, interactions, &user_map, interactions_stale),
            user_relationships,
        ))
    }
}
