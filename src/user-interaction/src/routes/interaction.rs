use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{middleware, Router};
use axum::routing::{delete, post};
use ulid::Ulid;
use crate::errors::AppError;
use crate::models::app_state::AppState;
use crate::services::{amq, redis};
use crate::auth::authorization_middleware;
use crate::models::amq_message::{PostInteractedMessage};
use crate::setting::Auth;
use crate::utils::constants::POST_EXCHANGE_NAME;

pub fn create_router(app_state: AppState, auth: Auth) -> Router {
    Router::new()
        .route("/api/users/:userId/likes/posts/:postId", post(like_post))
        .route("/api/users/:userId/likes/posts/:postId", delete(unlike_post))
        .route("/api/users/:userId/views/posts/:postId", post(view_post))
        .layer(middleware::from_fn_with_state(auth.clone(), authorization_middleware))
        .with_state(app_state)
}

async fn like_post(State(state): State<AppState>, Path((user_id, post_id)): Path<(Ulid,Ulid)>) -> Result<StatusCode, AppError> {
    redis::like_post(&state.redis, user_id.to_string(), post_id.to_string()).await?;
    
    amq::publish_event(&state.amq, POST_EXCHANGE_NAME,"post.liked", &PostInteractedMessage::new(post_id, user_id)).await?;
    Ok(StatusCode::CREATED)
}

async fn unlike_post(State(state): State<AppState>, Path((user_id, post_id)): Path<(Ulid,Ulid)>) -> Result<StatusCode, AppError> {
    redis::unlike_post(&state.redis, user_id.to_string(), post_id.to_string()).await?;
    
    amq::publish_event(&state.amq, POST_EXCHANGE_NAME,"post.unliked", &PostInteractedMessage::new(post_id, user_id)).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn view_post(State(state): State<AppState>, Path((user_id, post_id)): Path<(Ulid,Ulid)>) -> Result<StatusCode, AppError> {
    redis::add_view(&state.redis, user_id.to_string(), post_id.to_string()).await?;
    
    amq::publish_event(&state.amq, POST_EXCHANGE_NAME,"post.viewed", &PostInteractedMessage::new(post_id, user_id)).await?;
    Ok(StatusCode::CREATED)
}