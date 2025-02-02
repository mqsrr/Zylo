use crate::errors;
use crate::models::app_state::AppState;
use crate::repositories::interaction_repo::InteractionRepository;
use crate::services::amq_client::AmqClient;
use crate::services::post_interactions_service::PostInteractionsService;
use crate::services::reply_service::ReplyService;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, post};
use axum::Router;
use ulid::Ulid;

pub fn create_router<A, I, RS, PS>(state: AppState<A, I, RS, PS>) -> Router
where
    A: AmqClient + 'static,
    I: InteractionRepository + 'static,
    RS: ReplyService + 'static,
    PS: PostInteractionsService + 'static,
{
    Router::new()
        .route("/api/users/{userId}/likes/posts/{postId}", post(like_post))
        .route(
            "/api/users/{userId}/likes/posts/{postId}",
            delete(unlike_post),
        )
        .route("/api/users/{userId}/views/posts/{postId}", post(view_post))
        .with_state(state)
}

async fn like_post<A, I, RS, PS>(
    State(state): State<AppState<A, I, RS, PS>>,
    Path((user_id, post_id)): Path<(Ulid, Ulid)>,
) -> Result<StatusCode, errors::AppError>
where
    A: AmqClient + 'static,
    I: InteractionRepository + 'static,
    RS: ReplyService + 'static,
    PS: PostInteractionsService + 'static,
{
    let is_liked = state
        .interaction_repo
        .like(&post_id.to_string(), &user_id)
        .await?;

    match is_liked {
        true => Ok(StatusCode::CREATED),
        false =>  Err(errors::AppError::NotFound(String::from("Post or user could not be found"))),
    }
}

async fn unlike_post<A, I, RS, PS>(
    State(state): State<AppState<A, I, RS, PS>>,
    Path((user_id, post_id)): Path<(Ulid, Ulid)>,
) -> Result<StatusCode, errors::AppError>
where
    A: AmqClient + 'static,
    I: InteractionRepository + 'static,
    RS: ReplyService + 'static,
    PS: PostInteractionsService + 'static,
{
    let is_unliked = state
        .interaction_repo
        .unlike(&post_id.to_string(), &user_id)
        .await?;

    match is_unliked {
        true => Ok(StatusCode::NO_CONTENT),
        false => Err(errors::AppError::NotFound(String::from("Post or user could not be found"))),
    }
}

async fn view_post<A, I, RS, PS>(
    State(state): State<AppState<A, I, RS, PS>>,
    Path((user_id, post_id)): Path<(Ulid, Ulid)>,
) -> Result<StatusCode, errors::AppError>
where
    A: AmqClient + 'static,
    I: InteractionRepository + 'static,
    RS: ReplyService + 'static,
    PS: PostInteractionsService + 'static,
{
    let is_applied = state
        .interaction_repo
        .view(&post_id.to_string(), &user_id)
        .await?;

    match is_applied {
        true => Ok(StatusCode::CREATED),
        false => Ok(StatusCode::OK),
    }
}
