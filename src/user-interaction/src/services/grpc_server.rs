use crate::errors;
use crate::models::reply::{PostInteractionResponse, ReplyResponse};
use crate::services::grpc_server::reply_server::reply_service_server::ReplyService as GrpcReplyService;
use crate::services::grpc_server::reply_server::{BatchOfPostInteractionsResponse, GetBatchOfPostInteractionsRequest, GetPostInteractionsRequest, GetReplyByIdRequest, PostInteractionsResponse as GrpcPostInteractionsResponse, ReplyResponse as GrpcReplyResponse};
use crate::services::post_interactions_service::PostInteractionsService;
use crate::services::reply_service::ReplyService;
use chrono::Utc;
use std::str::FromStr;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use ulid::Ulid;

pub mod reply_server {
    tonic::include_proto!("reply_server");
}

impl From<ReplyResponse> for GrpcReplyResponse {
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
                .map(GrpcReplyResponse::from)
                .collect(),
            likes: reply_response.likes,
            views: reply_response.views,
            user_interacted: reply_response.user_interacted.unwrap_or(false),
        }
    }
}

impl From<PostInteractionResponse> for GrpcPostInteractionsResponse {
    fn from(value: PostInteractionResponse) -> Self {
        Self {
            post_id: value.post_id.to_string(),
            replies: value.replies.into_iter().map(GrpcReplyResponse::from).collect(),
            likes: value.likes,
            views: value.views,
            user_interacted: value.user_interacted.unwrap_or_default(),
        }
    }
}

#[derive(Debug)]
pub struct GrpcReplyServer<
    RS: ReplyService + 'static,
    PS: PostInteractionsService + 'static,
> {
    reply_service: Arc<RS>,
    post_interactions_service: Arc<PS>,
}

impl<
    RS,
    PS,
> GrpcReplyServer<RS, PS>
where
    RS: ReplyService + 'static,
    PS: PostInteractionsService + 'static
{
    pub fn new(reply_service: Arc<RS>, post_interactions_service: Arc<PS>) -> Self {
        Self {
            reply_service,
            post_interactions_service,
        }
    }
}

#[tonic::async_trait]
impl<
    RS,
    PS,
> GrpcReplyService for GrpcReplyServer<RS, PS>
where
    RS: ReplyService + 'static,
    PS: PostInteractionsService + 'static
{
    async fn get_reply_by_id(
        &self,
        request: Request<GetReplyByIdRequest>,
    ) -> Result<Response<GrpcReplyResponse>, Status> {
        let inner_request = request.into_inner();

        let reply_id = Ulid::from_str(inner_request.id.as_str())
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        let user_id = Ulid::from_str(inner_request.interaction_user_id.as_str())
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        let response = self
            .reply_service
            .get(&reply_id, Some(user_id))
            .await
            .map_err(|e| match e {
                errors::AppError::NotFound(err) => Status::not_found(err),
                _ => Status::internal(e.to_string()),
            })?;

        let grpc_reply = GrpcReplyResponse::from(response);
        Ok(Response::new(grpc_reply))
    }

    async fn get_post_interactions(
        &self,
        request: Request<GetPostInteractionsRequest>,
    ) -> Result<Response<GrpcPostInteractionsResponse>, Status> {
        let inner_request = request.into_inner();

        let post_id = Ulid::from_str(inner_request.post_id.as_str())
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        let interaction_user_id: Option<Ulid> =
            Ulid::from_str(&inner_request.interaction_user_id).ok();
        
        let response = self
            .post_interactions_service
            .get_post_interactions(post_id, interaction_user_id)
            .await
            .map_err(|e| match e {
                errors::AppError::NotFound(err) => Status::not_found(err),
                _ => Status::internal(e.to_string()),
            })?;

        Ok(Response::new(GrpcPostInteractionsResponse::from(response)))
    }

    async fn get_batch_of_post_interactions(
        &self,
        request: Request<GetBatchOfPostInteractionsRequest>,
    ) -> Result<Response<BatchOfPostInteractionsResponse>, Status> {
        let inner_request = request.into_inner();
        let posts_ids: Vec<Ulid> = inner_request
            .posts_ids
            .iter()
            .map(|id| Ulid::from_str(id).map_err(|e| Status::invalid_argument(e.to_string())))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| Status::internal(e.to_string()))?;

        let interaction_user_id: Option<Ulid> =
            Ulid::from_str(&inner_request.interaction_user_id).ok();

        let replies = self
            .post_interactions_service
            .get_posts_interactions(&posts_ids, interaction_user_id)
            .await
            .map_err(|e| match e {
                errors::AppError::NotFound(err) => Status::not_found(err),
                _ => Status::internal(e.to_string()),
            })?;

            Ok(Response::new(BatchOfPostInteractionsResponse {
                posts_interactions: replies.into_iter().map(GrpcPostInteractionsResponse::from).collect(),
            }))
        }
    }