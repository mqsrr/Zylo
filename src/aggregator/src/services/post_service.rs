use crate::errors;
use crate::models::post::{PaginatedResponse, Post};
use crate::services::InjectTraceContext;
use crate::services::aggregator::post_service_client::PostServiceClient;
use crate::services::aggregator::reply_service_client::ReplyServiceClient;
use crate::services::aggregator::user_profile_service_client::UserProfileServiceClient;
use crate::services::aggregator::{BatchOfPostInteractionsResponse, BatchPostsRequest, GetPostInteractionsRequest, PostInteractionsResponse, PostRequest, PostsRequest};
use async_trait::async_trait;
use std::collections::{HashMap};
use tonic::IntoRequest;
use tonic::transport::Channel;
use tracing::log::{error, warn};
use ulid::Ulid;
use crate::utils::helpers::{collect_user_ids_from_post, collect_user_ids_from_posts, fetch_user_summaries, get_posts_interactions};

#[async_trait]
pub trait PostsService: Send + Sync {
    async fn get_paginated_posts(
        &mut self,
        per_page: u32,
        interaction_user_id: Ulid,
        last_post_id: Option<String>,
    ) -> Result<PaginatedResponse<Post>, errors::GrpcError>;

    async fn get_post_by_id(
        &mut self,
        id: Ulid,
        interaction_user_id: Ulid,
    ) -> Result<Post, errors::GrpcError>;

    async fn get_posts_by_id(
        &mut self,
        id: Vec<String>,
        interaction_user_id: Ulid,
    ) -> Result<Vec<Post>, errors::GrpcError>;
}

pub struct PostsServiceImpl {
    post_client: PostServiceClient<Channel>,
    reply_client: ReplyServiceClient<Channel>,
    user_client: UserProfileServiceClient<Channel>,
}

impl PostsServiceImpl {
    pub fn new(
        post_client: PostServiceClient<Channel>,
        reply_client: ReplyServiceClient<Channel>,
        user_client: UserProfileServiceClient<Channel>,
    ) -> Self {
        Self {
            post_client,
            reply_client,
            user_client,
        }
    }
}

#[async_trait]
impl PostsService for PostsServiceImpl {
    async fn get_paginated_posts(
        &mut self,
        per_page: u32,
        interaction_user_id: Ulid,
        last_post_id: Option<String>,
    ) -> Result<PaginatedResponse<Post>, errors::GrpcError> {
        let request = PostsRequest {
            per_page: per_page as i32,
            last_post_id,
            user_id: None,
        }
        .into_request()
        .inject_trace_context();

        let paginated_posts = self
            .post_client
            .get_paginated_posts(request)
            .await?
            .into_inner();
        
        if paginated_posts.posts.is_empty() {
            return Ok(PaginatedResponse::new(Vec::new(), paginated_posts.per_page, paginated_posts.next_cursor, paginated_posts.has_next_page, false))
        }
        
        let mut interactions_stale = false;
        let interactions = get_posts_interactions(&mut self.reply_client, &paginated_posts.posts, interaction_user_id.to_string())
            .await
            .unwrap_or_else(|e| {
                error!("Failed to retrieve post interactions: {:?}", e);
                interactions_stale = true;
                BatchOfPostInteractionsResponse::default()
            });

        let user_ids = collect_user_ids_from_posts(&paginated_posts.posts, &interactions);
        let user_map = fetch_user_summaries(&mut self.user_client, user_ids)
            .await
            .unwrap_or_else(|e| {
                error!("Failed to retrieve user data: {:?}", e);
                HashMap::default()
            });

        Ok(PaginatedResponse::<Post>::from(
            paginated_posts,
            interactions,
            &user_map,
            interactions_stale
        ))
    }

    async fn get_post_by_id(
        &mut self,
        id: Ulid,
        interaction_user_id: Ulid,
    ) -> Result<Post, errors::GrpcError> {
        let post_id = id.to_string();
        let request = PostRequest {
            post_id: post_id.clone(),
        }
        .into_request()
        .inject_trace_context();

        let post_response = self.post_client.get_post_by_id(request).await?.into_inner();
        let request = GetPostInteractionsRequest {
            post_id,
            interaction_user_id: interaction_user_id.to_string(),
        }
        .into_request()
        .inject_trace_context();

        let interactions = self
            .reply_client
            .get_post_interactions(request)
            .await
            .map(|res| res.into_inner())
            .unwrap_or_else(|e| {
                warn!("Failed to retrieve post interactions: {:?}", e);
                PostInteractionsResponse::default()
            });

        let user_ids = collect_user_ids_from_post(&post_response, &interactions);
        let user_map = fetch_user_summaries(&mut self.user_client, user_ids)
            .await
            .unwrap_or_else(|e| {
                warn!("Failed to retrieve user data: {:?}", e);
                HashMap::default()
            });

        Ok(Post::from(post_response, interactions, &user_map))
    }

    async fn get_posts_by_id(
        &mut self,
        ids: Vec<String>,
        interaction_user_id: Ulid,
    ) -> Result<Vec<Post>, errors::GrpcError> {
        let request = BatchPostsRequest { post_ids: ids }
            .into_request()
            .inject_trace_context();

        let posts_response = self
            .post_client
            .get_batch_posts(request)
            .await?
            .into_inner();

        if posts_response.posts.is_empty() {
            return Ok(Vec::new())
        }

        let interactions = get_posts_interactions(&mut self.reply_client, &posts_response.posts, interaction_user_id.to_string())
            .await
            .unwrap_or_else(|e| {
                warn!("Failed to retrieve post interactions: {:?}", e);
                BatchOfPostInteractionsResponse::default()
            });

        let user_ids = collect_user_ids_from_posts(&posts_response.posts, &interactions);
        let users_map = fetch_user_summaries(&mut self.user_client, user_ids)
            .await
            .unwrap_or_else(|e| {
                warn!("Failed to retrieve user data: {:?}", e);
                HashMap::default()
            });

        Ok(Post::map_posts(
            posts_response.posts,
            interactions,
            &users_map,
        ))
    }
}
