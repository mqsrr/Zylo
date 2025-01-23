use crate::auth::authorization_middleware;
use crate::errors;
use crate::models::app_state::AppState;
use crate::models::event_messages::{PostCreatedMessage, PostDeletedMessage, PostUpdatedMessage};
use crate::models::post::PostResponse;
use crate::repositories::post_repo::PostRepository;
use crate::repositories::user_repo::UserRepository;
use crate::services::amq::AmqClient;
use crate::services::cache_service::CacheService;
use crate::utils::constants::POST_EXCHANGE_NAME;
use crate::utils::request::{
    ConstructableRequest, CreatePostRequest, PaginatedResponse, PaginationParams,
    UpdatePostRequest, Validate,
};
use async_trait::async_trait;
use axum::extract::{FromRequest, Path, Query, Request, State};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post, put};
use axum::{http::StatusCode, middleware, Json, Router};
use ulid::Ulid;

pub fn create_router<P,U, C, A>(app_state: AppState<P, U, C, A>) -> Router
where
    P: PostRepository + 'static,
    U: UserRepository + 'static,
    C: CacheService + 'static,
    A: AmqClient + 'static
{
    Router::new()
        .route("/api/posts", get(get_recent_posts))
        .route("/api/posts/:postId", get(get_post))
        .route(
            "/api/users/:userId/posts",
            post(create_post).get(get_users_posts),
        )
        .route(
            "/api/users/:userId/posts/:postId",
            put(update_post).delete(delete_post),
        )
        .layer(middleware::from_fn_with_state(
            app_state.config.auth.clone(),
            authorization_middleware,
        ))
        .with_state(app_state)
}

async fn create_post<P, U, C, A>(
    State(state): State<AppState<P,U, C, A>>,
    MultipartRequest(request): MultipartRequest<CreatePostRequest>,
) -> Result<(StatusCode, Json<PostResponse>), errors::AppError>
where
    P: PostRepository + 'static,
    U: UserRepository + 'static,
    C: CacheService + 'static,
    A: AmqClient + 'static,
{
    let user_exists = state.user_repo.exists(&request.user_id).await?;
    if !user_exists { 
        return  Err(errors::AppError::NotFound("User with provided id does not exists".to_string()));
    }
    
    let post = state.post_repo.create(request).await?;

    state
        .amq_client
        .publish_event(
            POST_EXCHANGE_NAME,
            "post.created",
            &PostCreatedMessage::from(&post),
        )
        .await?;

    Ok((StatusCode::OK, Json(PostResponse::from(post))))
}

pub async fn get_recent_posts<P, U,C, A>(
    Query(params): Query<PaginationParams>,
    State(state): State<AppState<P,U, C, A>>,
) -> Result<(StatusCode, Json<PaginatedResponse<PostResponse>>), errors::AppError>
where
    P: PostRepository + 'static,
    U: UserRepository + 'static,
    C: CacheService + 'static,
    A: AmqClient + 'static,
{
    let paginated_response = state
        .post_repo
        .get_paginated_posts(None, params.per_page, params.last_post_id)
        .await?;

    Ok((
        StatusCode::OK,
        Json(PaginatedResponse::from_page(
            paginated_response,
            PostResponse::from,
        )),
    ))
}

async fn get_post<P, U, C, A>(
    State(state): State<AppState<P,U, C, A>>,
    Path(post_id): Path<Ulid>,
) -> Result<(StatusCode, Json<PostResponse>), errors::AppError>
where
    P: PostRepository + 'static,
    U: UserRepository + 'static,
    C: CacheService + 'static,
    A: AmqClient + 'static,
{
    let post = state.post_repo.get(&post_id).await?;
    Ok((StatusCode::OK, Json(PostResponse::from(post))))
}

async fn get_users_posts<P,U, C, A>(
    Query(params): Query<PaginationParams>,
    State(state): State<AppState<P,U, C, A>>,
    Path(user_id): Path<Ulid>,
) -> Result<(StatusCode, Json<PaginatedResponse<PostResponse>>), errors::AppError>
where
    P: PostRepository + 'static,
    U: UserRepository + 'static,
    C: CacheService + 'static,
    A: AmqClient + 'static,
{
    let paginated_response = state
        .post_repo
        .get_paginated_posts(Some(user_id), params.per_page, params.last_post_id)
        .await?;
    

    Ok((
        StatusCode::OK,
        Json(PaginatedResponse::from_page(
            paginated_response,
            PostResponse::from,
        )),
    ))
}

async fn update_post<P, U, C, A>(
    State(state): State<AppState<P,U, C, A>>,
    MultipartRequest(request): MultipartRequest<UpdatePostRequest>,
) -> Result<(StatusCode, Json<PostResponse>), errors::AppError>
where
    P: PostRepository + 'static,
    U: UserRepository + 'static,
    C: CacheService + 'static,
    A: AmqClient + 'static,
{
    let updated_post = state.post_repo.update(request).await?;
    state
        .amq_client
        .publish_event(
            POST_EXCHANGE_NAME,
            "post.updated",
            &PostUpdatedMessage::from(&updated_post),
        )
        .await?;

    Ok((StatusCode::OK, Json(PostResponse::from(updated_post))))
}

async fn delete_post<P,U, C, A>(
    State(state): State<AppState<P,U, C, A>>,
    Path((user_id, post_id)): Path<(Ulid, Ulid)>,
) -> Result<StatusCode, errors::AppError>
where
    P: PostRepository + 'static,
    U: UserRepository + 'static,
    C: CacheService + 'static,
    A: AmqClient + 'static,
{
    state.post_repo.delete(&post_id).await?;

    state
        .cache_service
        .hdelete_all("users-posts", &format!("*{}*", user_id))
        .await?;
   
    state
        .amq_client
        .publish_event(
            POST_EXCHANGE_NAME,
            "post.deleted",
            &PostDeletedMessage::new(post_id, user_id),
        )
        .await?;

    Ok(StatusCode::NO_CONTENT)
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
        request.validate().map_err(|e| errors::AppError::from(e).into_response())?;

        Ok(Self(request))
    }
}
