use crate::errors;
use crate::models::app_state::AppState;
use crate::services::post_service::PostsService;
use crate::services::user_service::UserService;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use ulid::Ulid;
use crate::models::user::User;
use crate::services::feed_service::FeedService;

pub fn create_router<P, U, F>(state: AppState<P, U, F>) -> Router
where
    P: PostsService + 'static,
    U: UserService + 'static,
    F: FeedService + 'static,
{
    Router::new()
        .route("/api/users/{userId}", get(get_user))
        .with_state(state)
}

#[derive(Debug, Deserialize)]
pub struct GetUserParams {
    #[serde(rename = "interactionUserId")]
    pub interaction_user_id: Option<Ulid>,
}
async fn get_user<P, U, F>(
    State(state): State<AppState<P, U, F>>,
    Path(user_id): Path<Ulid>,
    Query(params): Query<GetUserParams>
) -> Result<(StatusCode, Json<User>), errors::AppError>
where
    P: PostsService + 'static,
    U: UserService + 'static,
    F: FeedService + 'static,
{
    let user = state.users_service.lock().await.get_by_id(user_id, params.interaction_user_id).await?;
    Ok((StatusCode::OK, Json(user)))
}
