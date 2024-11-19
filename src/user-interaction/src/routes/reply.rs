use crate::errors::{AppError, Validate};
use crate::models::app_state::AppState;
use crate::models::reply::Reply;
use crate::services::{amq, redis};
use crate::utils::request::{map_nested, CreateReplyRequest, PostInteractionResponse, PostInteractionResponseBuilder, ReplyResponse, UpdateReplyRequest};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{get, put};
use axum::{middleware, Json, Router};
use chrono::{Utc};
use log::{error};
use serde::Deserialize;
use ulid::Ulid;
use crate::auth::authorization_middleware;
use crate::models::amq_message::{ReplyCreatedMessage, ReplyDeletedMessage, ReplyUpdatedMessage};
use crate::setting::Auth;
use crate::utils::constants::POST_EXCHANGE_NAME;

pub fn create_router(app_state: AppState, auth: Auth) -> Router {
    Router::new()
        .route("/api/posts/:postId/replies", get(get_all_from_post).post(create_reply))
        .route("/api/posts/:postId/replies/:replyId", put(update_reply).delete(delete_reply))
        .layer(middleware::from_fn_with_state(auth.clone(), authorization_middleware))
        .with_state(app_state)
}

#[derive(Deserialize)]
struct GetAllFromPostParams {
    #[serde(rename="userId")]
    user_id: Option<Ulid>,
}

async fn get_all_from_post(State(mut app_state): State<AppState>, Path(post_id): Path<Ulid>, Query(params): Query<GetAllFromPostParams>) -> Result<(StatusCode, Json<PostInteractionResponse>), AppError> {
    let cache_field = params.user_id
        .map_or_else(|| format!("*{}*", post_id), |user_id| format!("*{}-{}*", user_id, post_id));

    if let Some(cached_value) = redis::hash_scan::<PostInteractionResponse>(&app_state.redis, "user-interaction:replies", &cache_field).await.unwrap_or_default() {
        return Ok((StatusCode::OK, Json(cached_value)));
    }
    
    let mut response_builder = PostInteractionResponseBuilder::new();
    let mut replies: Vec<ReplyResponse> = Reply::get_all_from_post(&app_state.db, post_id, &mut app_state.user_profile_service)
        .await?
        .into_iter()
        .map(ReplyResponse::from)
        .collect();
    
    let mut fields = replies.iter().map(|r| r.id.to_string()).collect::<Vec<_>>();
    fields.push(post_id.to_string());

    let likes = redis::get_interactions(&app_state.redis, "user-interaction:posts:likes", &fields).await?;
    let views = redis::get_interactions(&app_state.redis, "user-interaction:posts:views", &fields).await?;

    for reply in replies.iter_mut() {
        reply.likes = likes[&reply.id];
        reply.views = views[&reply.id];
    }
    
    if let Some(user_id) = params.user_id {
        let user_interactions = redis::is_user_interacted(&app_state.redis, &user_id.to_string(), fields).await?;
        for reply in replies.iter_mut() {
            reply.user_interacted = Some(*user_interactions.get(&reply.id.to_string()).unwrap_or(&false));
        }
        
        response_builder = response_builder.user_interacted(Some(*user_interactions.get(&post_id.to_string()).unwrap_or(&false)));
    }
    
    let mut ids = vec![cache_field];
    ids.extend(replies.iter().map(|r| r.id.to_string()));
    let cache_field = ids.join(",");
    
    let response = response_builder
        .post_id(post_id)
        .replies(map_nested(replies))
        .likes(*likes.get(&post_id).unwrap_or(&0))
        .views(*views.get(&post_id).unwrap_or(&0))
        .build();
    
    let expire = (app_state.config.redis.expire_time * 60) as i64;
    redis::hash_set(&app_state.redis, "user-interaction:replies", &cache_field, &response, expire).await.map_err(|e| error!("{}",e)).unwrap();
    Ok((StatusCode::OK, Json(response)))
}

async fn create_reply(State(mut app_state): State<AppState>, Path(post_id): Path<Ulid>, Json(request): Json<CreateReplyRequest>) -> Result<(StatusCode, Json<ReplyResponse>), AppError> {
    request.validate()?;
    let reply = Reply::from(request);

    let created_reply = Reply::create(&app_state.db, &mut app_state.user_profile_service, &reply).await?;
    let response = ReplyResponse::from(created_reply);

    redis::hash_delete_all(&app_state.redis, "user-interaction:replies", &format!("*{}*", &post_id)).await.map_err(|e| error!("{}", e)).unwrap();
    amq::publish_event(&app_state.amq, POST_EXCHANGE_NAME, "reply.created", &ReplyCreatedMessage::from(response.clone())).await?;
    
    Ok((StatusCode::OK, Json(response)))
}

async fn update_reply(State(mut app_state): State<AppState>, Path((post_id, reply_id)): Path<(Ulid, Ulid)>, Json(request): Json<UpdateReplyRequest>) -> Result<(StatusCode, Json<ReplyResponse>), AppError> {
    let updated_reply = Reply::update(&app_state.db, &mut app_state.user_profile_service, &reply_id, &request.content).await?;
    let response = ReplyResponse::from(updated_reply);

    redis::hash_delete_all(&app_state.redis, "user-interaction:replies", &format!("*{}*", &post_id)).await.map_err(|e| error!("{}", e)).unwrap();
    amq::publish_event(&app_state.amq, POST_EXCHANGE_NAME, "reply.updated", &ReplyUpdatedMessage::new(reply_id, request.content, Utc::now().naive_utc())).await?;
    
    Ok((StatusCode::OK, Json(response)))
}

async fn delete_reply(State(app_state): State<AppState>, Path((post_id, reply_id)): Path<(Ulid, Ulid)>) -> Result<StatusCode, AppError> {
    Reply::delete(&app_state.db, reply_id).await?;
    redis::delete_interactions(&app_state.redis, &post_id.to_string()).await?;

    amq::publish_event(&app_state.amq, POST_EXCHANGE_NAME, "reply.deleted", &ReplyDeletedMessage::new(reply_id)).await?;
    Ok(StatusCode::NO_CONTENT)
}