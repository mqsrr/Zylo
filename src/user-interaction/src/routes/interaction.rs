use crate::errors::AppError;
use crate::models::app_state::AppState;
use crate::repositories::interaction_repo::InteractionRepository;
use crate::repositories::reply_repo::ReplyRepository;
use crate::services::amq_client::AmqClient;
use crate::services::cache_service::CacheService;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, post};
use axum::Router;
use ulid::Ulid;

pub fn create_router<
    R: ReplyRepository + 'static,
    I: InteractionRepository + 'static,
    A: AmqClient + 'static,
    C: CacheService + 'static,
>(state: AppState<R, I, A, C>) -> Router {
    Router::new()
        .route("/api/users/{userId}/likes/posts/{postId}", post(like_post))
        .route(
            "/api/users/{userId}/likes/posts/{postId}",
            delete(unlike_post),
        )
        .route("/api/users/{userId}/views/posts/{postId}", post(view_post))
        .with_state(state)
}

async fn like_post<
    R: ReplyRepository + 'static,
    I: InteractionRepository + 'static,
    A: AmqClient + 'static,
    C: CacheService + 'static,
>(
    State(state): State<AppState<R, I, A, C>>,
    Path((user_id, post_id)): Path<(Ulid, Ulid)>,
) -> Result<StatusCode, AppError> {
    state
        .interaction_repo
        .like_post(user_id.to_string(), post_id.to_string())
        .await?;

    Ok(StatusCode::CREATED)
}

async fn unlike_post<
    R: ReplyRepository + 'static,
    I: InteractionRepository + 'static,
    A: AmqClient + 'static,
    C: CacheService + 'static,
>(
    State(state): State<AppState<R, I, A, C>>,
    Path((user_id, post_id)): Path<(Ulid, Ulid)>,
) -> Result<StatusCode, AppError> {
    state
        .interaction_repo.unlike_post(user_id.to_string(), post_id.to_string())
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn view_post<
    R: ReplyRepository + 'static,
    I: InteractionRepository + 'static,
    A: AmqClient + 'static,
    C: CacheService + 'static,
>(
    State(state): State<AppState<R, I, A, C>>,
    Path((user_id, post_id)): Path<(Ulid, Ulid)>,
) -> Result<StatusCode, AppError> {
    state
        .interaction_repo.add_view(user_id.to_string(), post_id.to_string())
        .await?;

    Ok(StatusCode::CREATED)
}
