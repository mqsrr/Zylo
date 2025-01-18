use crate::errors::AppError;
use crate::models::amq_message::{ReplyCreatedMessage, ReplyDeletedMessage, ReplyUpdatedMessage};
use crate::models::app_state::AppState;
use crate::models::reply::Reply;
use crate::repositories::interaction_repo::InteractionRepository;
use crate::repositories::reply_repo::ReplyRepository;
use crate::services::amq_client::AmqClient;
use crate::services::cache_service::CacheService;
use crate::utils::constants::POST_EXCHANGE_NAME;
use crate::utils::helpers::{build_cache_key, populate_interactions_for_replies};
use crate::utils::request::{
    map_nested, CreateReplyRequest, PostInteractionResponse, PostInteractionResponseBuilder,
    ReplyResponse, UpdateReplyRequest, Validate,
};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{get, put};
use axum::{Json, Router};
use chrono::Utc;
use serde::Deserialize;
use ulid::Ulid;

pub fn create_router<
    R: ReplyRepository + 'static,
    I: InteractionRepository + 'static,
    A: AmqClient + 'static,
    C: CacheService + 'static,
>(
    state: AppState<R, I, A, C>,
) -> Router {
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

async fn get_all_from_post<
    R: ReplyRepository + 'static,
    I: InteractionRepository + 'static,
    A: AmqClient + 'static,
    C: CacheService + 'static,
>(
    State(state): State<AppState<R, I, A, C>>,
    Path(post_id): Path<Ulid>,
    Query(params): Query<GetAllFromPostParams>,
) -> Result<(StatusCode, Json<PostInteractionResponse>), AppError> {
    let cache_field = build_cache_key(
        params.user_id.map(|u| u.to_string()),
        &post_id.to_string(),
    );
    if let Some(cached_value) = state
        .cache_service
        .hfind("user-interaction:replies", &cache_field)
        .await
        .unwrap_or_default()
    {
        return Ok((StatusCode::OK, Json(cached_value)));
    }

    let mut reply_responses: Vec<ReplyResponse> = state
        .reply_repo
        .get_all_from_post(&post_id)
        .await?
        .into_iter()
        .map(ReplyResponse::from)
        .collect();

    populate_interactions_for_replies(
        &mut reply_responses,
        state.interaction_repo.clone(),
        params.user_id,
    )
    .await?;

    let mut user_interacted_on_post = false;
    let post_id_string = post_id.to_string();
    if let Some(user_id) = params.user_id {
        user_interacted_on_post = state
            .interaction_repo
            .is_user_liked(&user_id.to_string(), &post_id_string)
            .await?;
    }

    let post_likes = state
        .interaction_repo
        .get_interaction("user-interaction:posts:likes", &post_id_string)
        .await?;

    let post_views = state
        .interaction_repo
        .get_interaction("user-interaction:posts:views", &post_id_string)
        .await?;

    let mut ids = vec![cache_field.replacen('*', "", 2)];
    ids.extend(reply_responses.iter().map(|r| r.id.to_string()));
    let cache_field = ids.join("-");
    
    let response = PostInteractionResponseBuilder::new()
        .post_id(post_id)
        .replies(map_nested(reply_responses))
        .likes(post_likes)
        .views(post_views)
        .user_interacted(Some(user_interacted_on_post))
        .build();
    
    state
        .cache_service
        .hset("user-interaction:replies", &cache_field, &response)
        .await?;

    Ok((StatusCode::OK, Json(response)))
}

async fn get_reply<
    R: ReplyRepository + 'static,
    I: InteractionRepository + 'static,
    A: AmqClient + 'static,
    C: CacheService + 'static,
>(
    State(state): State<AppState<R, I, A, C>>,
    Path((_, reply_id)): Path<(Ulid, Ulid)>,
    Query(params): Query<GetAllFromPostParams>,
) -> Result<(StatusCode, Json<ReplyResponse>), AppError> {
    let cache_field = build_cache_key(
        params.user_id.map(|u| u.to_string()),
        &reply_id.to_string(),
    );

    if let Some(cached_value) = state
        .cache_service
        .hfind("user-interaction:replies", &reply_id.to_string())
        .await
        .unwrap_or_default()
    {
        return Ok((StatusCode::OK, Json(cached_value)));
    }

    let prefix = state.reply_repo.get_reply_path(&reply_id).await?;
    let mut replies: Vec<ReplyResponse> = state
        .reply_repo
        .get_with_nested_by_path_prefix(&prefix)
        .await?
        .into_iter()
        .map(ReplyResponse::from)
        .collect();

    populate_interactions_for_replies(&mut replies, state.interaction_repo.clone(), params.user_id)
        .await?;

    let mut ids = vec![cache_field];
    ids.extend(replies.iter().map(|r| r.id.to_string()));
    let cache_field = ids.join("-");

    let response = map_nested(replies).pop().unwrap();
    state
        .cache_service
        .hset("user-interaction:replies", &cache_field, &response)
        .await?;

    Ok((StatusCode::OK, Json(response)))
}

async fn create_reply<
    R: ReplyRepository + 'static,
    I: InteractionRepository + 'static,
    A: AmqClient + 'static,
    C: CacheService + 'static,
>(
    State(state): State<AppState<R, I, A, C>>,
    Path(post_id): Path<Ulid>,
    Json(request): Json<CreateReplyRequest>,
) -> Result<(StatusCode, Json<ReplyResponse>), AppError> {
    request.validate()?;
    let user_exists = state.cache_service.sismember("created-users", &request.user_id.to_string()).await?;
    let post_exists = state.cache_service.sismember("created-posts", &post_id.to_string()).await?;
    if !user_exists || !post_exists {
        return Err(AppError::ValidationError(String::from("User or post does not exists")));
    }
    
    let new_reply_id = Ulid::new();
    let parent_id = request.reply_to_id;
    
    let parent_path = if parent_id == post_id {
        format!("/{}/", post_id)
    } else {
        state.reply_repo.get_reply_path(&parent_id).await?
    };

    let new_path = format!("{}{}/", parent_path, new_reply_id);
    let reply = Reply::from_create(request, new_reply_id, post_id, new_path);

    state.reply_repo.create(&reply).await?;
    let response = ReplyResponse::from(reply);

    state
        .cache_service
        .delete_all("user-interaction:replies", &format!("*{}*", &post_id))
        .await?;

    state
        .amq_client
        .publish_event(
            POST_EXCHANGE_NAME,
            "reply.created",
            &ReplyCreatedMessage::from(response.clone()),
        )
        .await?;

    Ok((StatusCode::OK, Json(response)))
}

async fn update_reply<
    R: ReplyRepository + 'static,
    I: InteractionRepository + 'static,
    A: AmqClient + 'static,
    C: CacheService + 'static,
>(
    State(state): State<AppState<R, I, A, C>>,
    Path((post_id, reply_id)): Path<(Ulid, Ulid)>,
    Json(request): Json<UpdateReplyRequest>,
) -> Result<(StatusCode, Json<ReplyResponse>), AppError> {
    let updated_reply = state.reply_repo.update(&reply_id, &request.content).await?;

    let response = ReplyResponse::from(updated_reply);
    state
        .cache_service
        .delete_all("user-interaction:replies", &format!("*{}*", &post_id))
        .await?;

    state
        .amq_client
        .publish_event(
            POST_EXCHANGE_NAME,
            "reply.updated",
            &ReplyUpdatedMessage::new(reply_id, request.content, Utc::now().naive_utc()),
        )
        .await?;

    Ok((StatusCode::OK, Json(response)))
}

async fn delete_reply<
    R: ReplyRepository + 'static,
    I: InteractionRepository + 'static,
    A: AmqClient + 'static,
    C: CacheService + 'static,
>(
    State(state): State<AppState<R, I, A, C>>,
    Path((post_id, reply_id)): Path<(Ulid, Ulid)>,
) -> Result<StatusCode, AppError> {
    state.reply_repo.delete(&reply_id).await?;
    state
        .interaction_repo
        .delete_interactions(&post_id.to_string())
        .await?;

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
