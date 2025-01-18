use crate::errors;
use crate::repositories::interaction_repo::InteractionRepository;
use crate::repositories::reply_repo::ReplyRepository;
use crate::services::cache_service::CacheService;
use crate::services::grpc_server::reply_server::grpc_reply_service_server::GrpcReplyService;
use crate::services::grpc_server::reply_server::{
    BatchFetchPostInteractionsRequest, BatchFetchPostInteractionsResponse,
    FetchPostInteractionsRequest, FetchPostInteractionsResponse, FetchReplyByIdRequest,
    FetchReplyByIdResponse, GrpcPostInteraction, GrpcReply,
};
use crate::utils::helpers::{build_cache_key, populate_interactions_for_replies};
use crate::utils::request::{
    map_nested, PostInteractionResponse, PostInteractionResponseBuilder, ReplyResponse,
};
use std::str::FromStr;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::{warn};
use ulid::Ulid;

pub mod reply_server {
    tonic::include_proto!("reply_server");
    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("reply_server_descriptor");
}

#[derive(Debug)]
pub struct ReplyServer<
    R: ReplyRepository + 'static,
    I: InteractionRepository + 'static,
    C: CacheService + 'static,
> {
    reply_repo: Arc<R>,
    interaction_repo: Arc<I>,
    cache_service: Arc<C>,
}

impl<
        R: ReplyRepository + 'static,
        I: InteractionRepository + 'static,
        C: CacheService + 'static,
    > ReplyServer<R, I, C>
{
    pub fn new(reply_repo: Arc<R>, interaction_repo: Arc<I>, cache_service: Arc<C>) -> Self {
        Self {
            reply_repo,
            interaction_repo,
            cache_service,
        }
    }
}

#[tonic::async_trait]
impl<
        R: ReplyRepository + 'static,
        I: InteractionRepository + 'static,
        C: CacheService + 'static,
    > GrpcReplyService for ReplyServer<R, I, C>
{
    async fn fetch_reply_by_id(
        &self,
        request: Request<FetchReplyByIdRequest>,
    ) -> Result<Response<FetchReplyByIdResponse>, Status> {
        let inner_request = request.into_inner();

        let reply_id = Ulid::from_str(inner_request.id.as_str())
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        let user_id = Ulid::from_str(inner_request.id.as_str())
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        let cache_field = build_cache_key(Some(user_id.to_string()), &reply_id.to_string());

        if let Some(cached_reply_response) = self
            .cache_service
            .hfind::<ReplyResponse>("user-interaction:replies", &cache_field)
            .await
            .unwrap_or_default()
        {
            let grpc_reply = GrpcReply::from(cached_reply_response);
            return Ok(Response::new(FetchReplyByIdResponse {
                reply: Some(GrpcReply::from(grpc_reply)),
            }));
        }

        let replies = self
            .reply_repo
            .get_with_nested(&reply_id)
            .await
            .map_err(|e| match e {
                errors::DatabaseError::NotFound(_) => Status::not_found(e.to_string()),
                _ => Status::internal(e.to_string()),
            })?;

        let mut replies_responses: Vec<ReplyResponse> =
            replies.into_iter().map(ReplyResponse::from).collect();

        populate_interactions_for_replies(
            replies_responses.as_mut_slice(),
            self.interaction_repo.clone(),
            None,
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        let mut ids = vec![cache_field];
        ids.extend(replies_responses.iter().map(|r| r.id.to_string()));
        let cache_field = ids.join("-");

        let reply_response = map_nested(replies_responses)
            .pop()
            .ok_or_else(|| Status::not_found("Reply not found"))?;

        self.cache_service
            .hset("user-interaction:replies", &cache_field, &reply_response)
            .await
            .unwrap_or_else(|e| warn!("Could not create cache for reply: {:?}", e));

        let grpc_reply = GrpcReply::from(reply_response);
        Ok(Response::new(FetchReplyByIdResponse {
            reply: Some(grpc_reply),
        }))
    }

    async fn fetch_post_interactions(
        &self,
        request: Request<FetchPostInteractionsRequest>,
    ) -> Result<Response<FetchPostInteractionsResponse>, Status> {
        let inner_request = request.into_inner();

        let post_id = Ulid::from_str(inner_request.post_id.as_str())
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        let interaction_user_id: Option<Ulid> =
            Ulid::from_str(&inner_request.interaction_user_id).ok();

        let cache_field = build_cache_key(
            interaction_user_id.map(|u| u.to_string()),
            &post_id.to_string(),
        );
        if let Some(cached_interaction_response) = self
            .cache_service
            .hfind::<PostInteractionResponse>("user-interaction:replies", &cache_field)
            .await
            .unwrap_or_default()
        {
            return Ok(Response::new(FetchPostInteractionsResponse::from(
                cached_interaction_response,
            )));
        }

        let mut reply_responses: Vec<ReplyResponse> = self
            .reply_repo
            .get_all_from_post(&post_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .into_iter()
            .map(ReplyResponse::from)
            .collect();

        populate_interactions_for_replies(
            &mut reply_responses,
            self.interaction_repo.clone(),
            interaction_user_id,
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        let mut user_interacted_on_post = false;
        let post_id_string = post_id.to_string();
        if let Some(user_id) = interaction_user_id {
            user_interacted_on_post = self
                .interaction_repo
                .is_user_liked(&user_id.to_string(), &post_id_string)
                .await
                .map_err(|e| Status::internal(e.to_string()))?;
        }

        let post_likes = self
            .interaction_repo
            .get_interaction("user-interaction:posts:likes", &post_id_string)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let post_views = self
            .interaction_repo
            .get_interaction("user-interaction:posts:views", &post_id_string)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let response = PostInteractionResponseBuilder::new()
            .post_id(post_id)
            .replies(map_nested(reply_responses))
            .likes(post_likes)
            .views(post_views)
            .user_interacted(Some(user_interacted_on_post))
            .build();

        self.cache_service
            .hset("user-interaction:replies", &cache_field, &response)
            .await
            .unwrap_or_else(|e| warn!("Could not create cache for reply: {:?}", e));

        Ok(Response::new(FetchPostInteractionsResponse::from(response)))
    }

    async fn batch_fetch_post_interactions(
        &self,
        request: Request<BatchFetchPostInteractionsRequest>,
    ) -> Result<Response<BatchFetchPostInteractionsResponse>, Status> {
        let inner_request = request.into_inner();
        let post_ids: Vec<Ulid> = inner_request
            .posts_ids
            .iter()
            .map(|id| Ulid::from_str(id).map_err(|e| Status::invalid_argument(e.to_string())))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| Status::internal(e.to_string()))?;

        let interaction_user_id: Option<Ulid> =
            Ulid::from_str(&inner_request.interaction_user_id).ok();

        let grouped_replies_map = self
            .reply_repo
            .get_all_from_posts(&post_ids)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let post_ids_str: Vec<String> = post_ids.iter().map(|id| id.to_string()).collect();
        let post_likes = self
            .interaction_repo
            .get_interactions("user-interaction:posts:likes", &post_ids_str)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let post_views = self
            .interaction_repo
            .get_interactions("user-interaction:posts:views", &post_ids_str)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let mut grouped_replies: Vec<GrpcPostInteraction> = Vec::new();
        for (post_id, replies) in grouped_replies_map {
            let post_id_str = post_id.to_string();

            let grpc_replies: Vec<GrpcReply> =
                map_nested(replies.into_iter().map(ReplyResponse::from).collect())
                    .into_iter()
                    .map(GrpcReply::from)
                    .collect();

            let user_interacted = match &interaction_user_id {
                Some(user_id) => self
                    .interaction_repo
                    .is_user_liked(&user_id.to_string(), &post_id_str)
                    .await
                    .unwrap_or(false),
                None => false,
            };

            grouped_replies.push(GrpcPostInteraction {
                post_id: post_id_str.clone(),
                replies: grpc_replies,
                likes: *post_likes.get(&post_id).unwrap_or(&0),
                views: *post_views.get(&post_id).unwrap_or(&0),
                user_interacted,
            });
        }

        Ok(Response::new(BatchFetchPostInteractionsResponse {
            posts_interactions: grouped_replies,
        }))
    }
}