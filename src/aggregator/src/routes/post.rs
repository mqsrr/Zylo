use crate::errors;
use crate::models::app_state::AppState;
use crate::models::post::{PaginatedResponse, Post};
use crate::services::post_service::PostsService;
use crate::services::user_service::UserService;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use ulid::Ulid;
use crate::services::feed_service::FeedService;

pub fn create_router<P, U, F>(state: AppState<P, U, F>) -> Router
where
    P: PostsService + 'static,
    U: UserService + 'static,
    F: FeedService + 'static,
{
    Router::new()
        .route("/api/posts/{postId}", get(get_post))
        .route("/api/posts", get(get_recent_posts))
        .route("/api/users/{userId}/feed", get(get_feed))
        .with_state(state)
}

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    #[serde(rename = "userInteractionId")]
    pub user_interaction_id: Ulid,
    #[serde(rename = "next")]
    pub next: Option<Ulid>,
    #[serde(rename = "perPage")]
    pub per_page: Option<u32>,
}

pub async fn get_recent_posts<P, U, F>(
    Query(params): Query<PaginationParams>,
    State(state): State<AppState<P, U, F>>,
) -> Result<(StatusCode, Json<PaginatedResponse<Post>>), errors::AppError>
where
    P: PostsService + 'static,
    U: UserService + 'static,
    F: FeedService + 'static,
{
    let paginated_response = state
        .posts_service
        .lock()
        .await
        .get_paginated_posts(
            params.per_page.unwrap_or(10),
            params.user_interaction_id,
            params.next.map(|next| next.to_string()),
        )
        .await?;

    Ok((StatusCode::OK, Json(paginated_response)))
}

#[derive(Debug, Deserialize)]
pub struct GetPostByIdQueryParams {
    #[serde(rename = "userInteractionId")]
    pub user_interaction_id: Ulid,
}


async fn get_post<P, U, F>(
    State(state): State<AppState<P, U, F>>,
    Path(post_id): Path<Ulid>,
    Query(params): Query<GetPostByIdQueryParams>,
) -> Result<(StatusCode, Json<Post>), errors::AppError>
where
    P: PostsService + 'static,
    U: UserService + 'static,
    F: FeedService + 'static,
{
    let post = state.posts_service.lock().await.get_post_by_id(post_id,params.user_interaction_id).await?;
    Ok((StatusCode::OK, Json(post)))
}

#[derive(Debug, Deserialize)]
pub struct FeedParams {
    #[serde(rename = "next")]
    pub next: Option<Ulid>,
    #[serde(rename = "perPage")]
    pub per_page: Option<u32>,
}

async fn get_feed<P, U, F>(
    State(state): State<AppState<P, U, F>>,
    Path(user_id): Path<Ulid>,
    Query(params): Query<FeedParams>,
) -> Result<(StatusCode, Json<PaginatedResponse<Post>>), errors::AppError>
where
    P: PostsService + 'static,
    U: UserService + 'static,
    F: FeedService + 'static,
{
    let post = state.feed_service.lock().await.get_feed_by_user_id(user_id, params.per_page, params.next.map(|id| id.to_string())).await?;
    Ok((StatusCode::OK, Json(post)))
}
