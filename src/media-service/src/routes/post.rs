use crate::auth::authorization_middleware;
use crate::errors::{AppError, Validate};
use crate::models::app_state::AppState;
use crate::models::event_messages::{PostCreatedMessage, PostDeletedMessage, PostUpdatedMessage};
use crate::models::post::{Post, PostResponse};
use crate::models::user::User;
use crate::services::s3::S3Service;
use crate::services::{amq, redis};
use crate::utils::constants::POST_EXCHANGE_NAME;
use crate::utils::requests::{
    ConstructableRequest, CreatePostRequest, PaginatedResponse, PaginationParams, UpdatePostRequest,
};
use async_trait::async_trait;
use axum::extract::{FromRequest, Path, Query, Request, State};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post, put};
use axum::{http::StatusCode, middleware, Json, Router};
use ulid::Ulid;

pub fn create_router<S: S3Service + Send + Sync + Clone + 'static>(
    app_state: AppState<S>,
) -> Router {
    Router::new()
        .route("/api/posts", get(get_recent_posts))
        .route("/api/posts/:postId", get(get_post))
        .route("/api/users/:userId/posts", post(create_post))
        .route("/api/users/:userId/posts", get(get_users_posts))
        .route("/api/users/:userId/posts/:postId", put(update_post))
        .route("/api/users/:userId/posts/:postId", delete(delete_post))
        .route(
            "/api/users/:userId/posts/:postId/media/:mediaId",
            get(get_presigned_url),
        )
        .layer(middleware::from_fn_with_state(
            app_state.config.auth.clone(),
            authorization_middleware,
        ))
        .with_state(app_state)
}

async fn create_post<S: S3Service>(
    State(mut state): State<AppState<S>>,
    MultipartRequest(request): MultipartRequest<CreatePostRequest>,
) -> Result<(StatusCode, Json<PostResponse>), AppError> {
    let post = Post::create(request, &state.db, state.s3file_service.clone()).await?;
    let user = User::get(post.user_id, &state.db, &mut state.user_profile_service).await?;

    redis::hash_delete(&state.redis, "media", &post.id.to_string()).await?;
    redis::hash_delete(&state.redis, "media", &post.user_id.to_string()).await?;

    amq::publish_event(
        &state.amq,
        POST_EXCHANGE_NAME,
        "post.created",
        &PostCreatedMessage::from(&post),
    )
        .await?;

    Ok((StatusCode::OK, Json(PostResponse::from(post, user))))
}

pub async fn get_recent_posts<S: S3Service>(
    Query(params): Query<PaginationParams>,
    State(mut state): State<AppState<S>>,
) -> Result<(StatusCode, Json<PaginatedResponse<PostResponse>>), AppError> {
    let posts = Post::get_posts(
        &state.db,
        state.s3file_service.clone(),
        None,
        params.last_created_at,
        params.per_page,
    ).await?;

    let mut post_responses = Vec::new();

    for post in posts {
        match User::get(post.user_id, &state.db, &mut state.user_profile_service).await {
            Ok(user) => {
                let post_response = PostResponse::from(post, user);
                post_responses.push(post_response);
            },
            Err(e) => {
                eprintln!("Error fetching user for post {}: {:?}", post.user_id, e);
            }
        }
    }
    let per_page = params.per_page.unwrap_or(10);

    let has_next_page = post_responses.len() == per_page as usize;
    let next_cursor = post_responses
        .last()
        .map(|post| post.created_at.clone())
        .unwrap_or_default();

    Ok((StatusCode::OK, Json(PaginatedResponse::new(post_responses, per_page, has_next_page, next_cursor))))
}

async fn get_post<S: S3Service>(
    State(mut state): State<AppState<S>>,
    Path(post_id): Path<Ulid>,
) -> Result<(StatusCode, Json<PostResponse>), AppError> {
    if let Some(post) =
        redis::hash_get::<PostResponse>(&state.redis, "media", &post_id.to_string()).await?
    {
        return Ok((StatusCode::OK, Json(post.into())));
    }

    let post = Post::get(post_id, &state.db, state.s3file_service.clone()).await?;
    let user = User::get(post.user_id, &state.db, &mut state.user_profile_service).await?;

    let post_response = PostResponse::from(post, user);
    let expire = (state.config.redis.expire_time * 60) as i64;

    redis::hash_set(
        &state.redis,
        "media",
        &post_id.to_string(),
        &post_response,
        expire,
    )
        .await?;

    Ok((StatusCode::OK, Json(post_response)))
}

async fn get_users_posts<S: S3Service>(
    Query(params): Query<PaginationParams>,
    State(mut state): State<AppState<S>>,
    Path(user_id): Path<Ulid>,
) -> Result<(StatusCode, Json<PaginatedResponse<PostResponse>>), AppError> {
    if let Some(posts) = redis::hash_get::<PaginatedResponse<PostResponse>>(&state.redis, "media", &user_id.to_string()).await? {
        return Ok((StatusCode::OK, Json(posts)));
    }

    let user = User::get(user_id, &state.db, &mut state.user_profile_service).await?;
    let posts = Post::get_posts(
        &state.db,
        state.s3file_service.clone(),
        Some(user_id),
        params.last_created_at,
        params.per_page,
    ).await?;


    let expire = state.config.s3_config.expire_time * 60;
    redis::hash_set(
        &state.redis,
        "media",
        &user_id.to_string(),
        &posts,
        expire as i64,
    )
        .await?;
    Ok((StatusCode::OK, Json(PaginatedResponse::from_posts(posts, user, params.per_page))))
}

async fn update_post<S: S3Service>(
    State(mut state): State<AppState<S>>,
    MultipartRequest(request): MultipartRequest<UpdatePostRequest>,
) -> Result<(StatusCode, Json<PostResponse>), AppError> {
    let post = Post::update(request, &state.db, state.s3file_service.clone()).await?;
    let user = User::get(post.user_id, &state.db, &mut state.user_profile_service).await?;

    redis::hash_delete(&state.redis, "media", &post.id.to_string()).await?;
    redis::hash_delete(&state.redis, "media", &post.user_id.to_string()).await?;

    amq::publish_event(
        &state.amq,
        POST_EXCHANGE_NAME,
        "post.updated",
        &PostUpdatedMessage::from(&post),
    )
        .await?;

    Ok((StatusCode::OK, Json(PostResponse::from(post, user))))
}

async fn delete_post<S: S3Service>(
    State(state): State<AppState<S>>,
    Path((user_id, post_id)): Path<(Ulid, Ulid)>,
) -> Result<StatusCode, AppError> {
    Post::delete(post_id, &state.db, state.s3file_service.clone()).await?;

    redis::hash_delete(&state.redis, "media", &post_id.to_string()).await?;
    redis::hash_delete(&state.redis, "media", &user_id.to_string()).await?;

    amq::publish_event(
        &state.amq,
        POST_EXCHANGE_NAME,
        "post.deleted",
        &PostDeletedMessage::new(post_id, user_id),
    )
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_presigned_url<S: S3Service>(
    State(state): State<AppState<S>>,
    Path((_, _, media_id)): Path<(Ulid, Ulid, Ulid)>,
) -> Result<impl IntoResponse, AppError> {
    let url = state
        .s3file_service
        .get_presigned_url_for_download(&media_id.to_string())
        .await?;

    Ok(url.url)
}

struct MultipartRequest<T: ConstructableRequest + Validate>(T);

#[async_trait]
impl<S, T> FromRequest<S> for MultipartRequest<T>
where
    S: Send + Sync,
    T: ConstructableRequest + Validate,
{
    type Rejection = Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let request = T::parse(req, state).await.map_err(|e| e.into_response())?;
        request.validate().map_err(|e| e.into_response())?;

        Ok(Self(request))
    }
}
