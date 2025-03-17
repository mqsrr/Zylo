use crate::errors;
use crate::models::post::{PaginatedResponse, Post};
use crate::services::aggregator::feed_service_client::FeedServiceClient;
use crate::services::aggregator::GetRecommendedPostsRequest;
use crate::services::post_service::PostsService;
use crate::services::InjectTraceContext;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::transport::Channel;
use tonic::IntoRequest;
use ulid::Ulid;

#[async_trait]
pub trait FeedService: Send + Sync {
    async fn get_feed_by_user_id(
        &mut self,
        id: Ulid,
        per_page: Option<u32>,
        last_post_id: Option<String>,
    ) -> Result<PaginatedResponse<Post>, errors::GrpcError>;
}

pub struct FeedServiceImpl<P: PostsService + 'static> {
    feed_client: FeedServiceClient<Channel>,
    posts_service: Arc<Mutex<P>>,
}

impl<P: PostsService + 'static> FeedServiceImpl<P> {
    pub fn new(feed_client: FeedServiceClient<Channel>, posts_service: Arc<Mutex<P>>) -> Self {
        Self {
            feed_client,
            posts_service,
        }
    }
}

#[async_trait]
impl<P: PostsService + 'static> FeedService for FeedServiceImpl<P> {
    async fn get_feed_by_user_id(
        &mut self,
        id: Ulid,
        per_page: Option<u32>,
        last_post_id: Option<String>,
    ) -> Result<PaginatedResponse<Post>, errors::GrpcError> {
        let request = GetRecommendedPostsRequest {
            user_id: id.to_string(),
            last_post_id,
            per_page,
            min_likes: None,
        }
        .into_request()
        .inject_trace_context();

        let recommended_posts = self
            .feed_client
            .get_posts_recommendations(request)
            .await?
            .into_inner();
        
        if recommended_posts.post_ids.is_empty() {
            return Ok(PaginatedResponse::new(Vec::new(), recommended_posts.per_page, recommended_posts.next, recommended_posts.has_next_page, false))
        }

        let posts = self
            .posts_service
            .lock()
            .await
            .get_posts_by_id(recommended_posts.post_ids, id)
            .await?;
        
        Ok(PaginatedResponse::<Post>::new(
            posts,
            recommended_posts.per_page,
            recommended_posts.next,
            recommended_posts.has_next_page,
            false
        ))
    }
}
