use crate::errors::AppError;
use crate::models::amq_message::{ReplyCreatedMessage, ReplyDeletedMessage, ReplyUpdatedMessage};
use crate::models::app_state::AppState;
use crate::models::reply::{
    CreateReplyRequest, PostInteractionResponse, ReplyResponse, UpdateReplyRequest,
};
use crate::repositories::interaction_repo::InteractionRepository;
use crate::services::amq_client::AmqClient;
use crate::services::post_interactions_service::PostInteractionsService;
use crate::services::reply_service::ReplyService;
use crate::utils::constants::POST_EXCHANGE_NAME;
use crate::utils::helpers::Validate;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{get, put};
use axum::{Json, Router};
use chrono::Utc;
use serde::Deserialize;
use ulid::Ulid;

pub fn create_router<A, I, RS, PS>(state: AppState<A, I, RS, PS>) -> Router
where
    A: AmqClient + 'static,
    I: InteractionRepository + 'static,
    RS: ReplyService + 'static,
    PS: PostInteractionsService + 'static,
{
    Router::new()
        .route(
            "/api/posts/{postId}/replies",
            get(get_all_from_post).post(create_reply),
        )
        .route(
            "/api/posts/{postId}/replies/{replyId}",
            put(update_reply).delete(delete_reply).get(get_reply),
        )
        .with_state(state)
}

#[derive(Deserialize)]
struct GetAllFromPostParams {
    #[serde(rename = "userId")]
    user_id: Option<Ulid>,
}

async fn get_all_from_post<A, I, RS, PS>(
    State(state): State<AppState<A, I, RS, PS>>,
    Path(post_id): Path<Ulid>,
    Query(params): Query<GetAllFromPostParams>,
) -> Result<(StatusCode, Json<PostInteractionResponse>), AppError>
where
    A: AmqClient + 'static,
    I: InteractionRepository + 'static,
    RS: ReplyService + 'static,
    PS: PostInteractionsService + 'static,
{
    let response = state
        .post_interactions_service
        .get_post_interactions(post_id, params.user_id)
        .await?;

    Ok((StatusCode::OK, Json(response)))
}

async fn get_reply<A, I, RS, PS>(
    State(state): State<AppState<A, I, RS, PS>>,
    Path((_, reply_id)): Path<(Ulid, Ulid)>,
    Query(params): Query<GetAllFromPostParams>,
) -> Result<(StatusCode, Json<ReplyResponse>), AppError>
where
    A: AmqClient + 'static,
    I: InteractionRepository + 'static,
    RS: ReplyService + 'static,
    PS: PostInteractionsService + 'static,
{
    let response = state.reply_service.get(&reply_id, params.user_id).await?;
    Ok((StatusCode::OK, Json(response)))
}

async fn create_reply<A, I, RS, PS>(
    State(state): State<AppState<A, I, RS, PS>>,
    Path(post_id): Path<Ulid>,
    Json(request): Json<CreateReplyRequest>,
) -> Result<(StatusCode, Json<ReplyResponse>), AppError>
where
    A: AmqClient + 'static,
    I: InteractionRepository + 'static,
    RS: ReplyService + 'static,
    PS: PostInteractionsService + 'static,
{
    request.validate()?;
    let reply_response: ReplyResponse = state
        .reply_service
        .create(
            post_id,
            request.reply_to_id,
            &request.content,
            request.user_id,
        )
        .await?;

    state
        .amq_client
        .publish_event(
            POST_EXCHANGE_NAME,
            "reply.created",
            &ReplyCreatedMessage::from(reply_response.clone()),
        )
        .await?;

    Ok((StatusCode::OK, Json(reply_response)))
}

async fn update_reply<A, I, RS, PS>(
    State(state): State<AppState<A, I, RS, PS>>,
    Path((_, reply_id)): Path<(Ulid, Ulid)>,
    Json(request): Json<UpdateReplyRequest>,
) -> Result<(StatusCode, Json<ReplyResponse>), AppError>
where
    A: AmqClient + 'static,
    I: InteractionRepository + 'static,
    RS: ReplyService + 'static,
    PS: PostInteractionsService + 'static,
{
    let updated_reply = state
        .reply_service
        .update(&reply_id, &request.content)
        .await?;

    state
        .amq_client
        .publish_event(
            POST_EXCHANGE_NAME,
            "reply.updated",
            &ReplyUpdatedMessage::new(reply_id, request.content, Utc::now().naive_utc()),
        )
        .await?;

    Ok((StatusCode::OK, Json(updated_reply)))
}

async fn delete_reply<A, I, RS, PS>(
    State(state): State<AppState<A, I, RS, PS>>,
    Path((_, reply_id)): Path<(Ulid, Ulid)>,
) -> Result<StatusCode, AppError>
where
    A: AmqClient + 'static,
    I: InteractionRepository + 'static,
    RS: ReplyService + 'static,
    PS: PostInteractionsService + 'static,
{
    state.reply_service.delete(&reply_id).await?;

    state
        .amq_client
        .publish_event(
            POST_EXCHANGE_NAME,
            "reply.deleted",
            &ReplyDeletedMessage::from(reply_id),
        )
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
