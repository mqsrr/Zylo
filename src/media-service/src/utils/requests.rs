use crate::errors::{AppError, Validate};
use crate::models::file::File;
use crate::models::post::{Post, PostResponse};
use crate::models::user::User;
use async_trait::async_trait;
use axum::extract::{FromRequest, Multipart, Request};
use log::warn;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[async_trait]
pub trait ConstructableRequest {
    async fn parse<S: Send + Sync>(request: Request, state: &S) -> Result<Self, AppError>
    where
        Self: Sized;
}

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    #[serde(rename = "lastCreatedAt")]
    pub last_created_at: Option<String>,
    #[serde(rename = "pageSize")]
    pub per_page: Option<u16>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    #[serde(rename = "perPage")]
    pub per_page: u16,
    #[serde(rename = "hasNextPage")]
    pub has_next_page: bool,
    #[serde(rename = "next")]
    pub next_cursor: String,
}

impl<T> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, per_page: u16, has_next_page: bool, next_cursor: String) -> Self {
        Self {
            data,
            per_page,
            has_next_page,
            next_cursor,
        }
    }
}

impl PaginatedResponse<PostResponse> {
    pub fn from_posts(value: Vec<Post>, user: User, per_page: Option<u16>) -> Self {
        let post_responses: Vec<PostResponse> = value.into_iter().map(|post| PostResponse::from(post, user.clone())).collect();
        let per_page = per_page.unwrap_or(10);

        let has_next_page = post_responses.len() == per_page as usize;
        let next_cursor = post_responses
            .last()
            .map(|post| post.created_at.clone())
            .unwrap_or_default();

        Self {
            data: post_responses,
            per_page,
            has_next_page,
            next_cursor,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreatePostRequest {
    pub id: Ulid,
    pub user_id: Ulid,
    pub text: String,
    pub files: Vec<File>,
    pub created_at: String,
    pub updated_at: String,
}

impl CreatePostRequest {
    pub fn new(user_id: Ulid) -> Self {
        Self {
            id: Ulid::new(),
            user_id,
            text: String::new(),
            files: Vec::new(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub async fn from_multipart(mut multipart: Multipart, user_id: Ulid) -> Result<Self, AppError> {
        let mut request = Self::new(user_id);

        while let Some(field) = multipart
            .next_field()
            .await
            .map_err(|_| AppError::InvalidMultipartContent)?
        {
            match field.name() {
                Some("text") => request.text = field.text().await.unwrap_or_default(),
                Some("media") => {
                    let file = File::from_field(field).await?;
                    request.files.push(file);
                }
                _ => warn!("Unknown field"),
            }
        }
        Ok(request)
    }
}

#[async_trait]
impl ConstructableRequest for CreatePostRequest {
    async fn parse<S: Send + Sync>(req: Request, state: &S) -> Result<Self, AppError>
    where
        Self: Sized,
    {
        let user_id = extract_user_id(&req)?;
        let multipart = Multipart::from_request(req, &state)
            .await
            .map_err(|_| AppError::InvalidMultipartContent)?;

        CreatePostRequest::from_multipart(multipart, user_id).await
    }
}
fn extract_user_id(req: &Request) -> Result<Ulid, AppError> {
    let uri_path = req.uri().path();
    let user_id_str = uri_path
        .split('/')
        .nth(3)
        .ok_or_else(|| AppError::InvalidUri("Could not find user id".to_string()))?;

    Ulid::from_string(user_id_str).map_err(|_| AppError::InvalidUserId)
}

fn extract_post_id(req: &Request) -> Result<Ulid, AppError> {
    let uri_path = req.uri().path();
    let post_id_str = uri_path
        .split('/')
        .nth(5)
        .ok_or_else(|| AppError::InvalidUri("Could not find post id".to_string()))?;

    Ulid::from_string(post_id_str).map_err(|_| AppError::InvalidPostId)
}

impl Validate for CreatePostRequest {
    fn validate(&self) -> Result<(), AppError> {
        if self.text.trim().is_empty() {
            return Err(AppError::ValidationError(
                "Text field is required.".to_string(),
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct UpdatePostRequest {
    pub id: Ulid,
    pub user_id: Ulid,
    pub text: String,
    pub files: Vec<File>,
}

impl UpdatePostRequest {
    pub async fn from_multipart(
        mut multipart: Multipart,
        user_id: Ulid,
        post_id: Ulid,
    ) -> Result<Self, AppError> {
        let mut request = Self {
            id: post_id,
            user_id,
            text: String::new(),
            files: Vec::new(),
        };

        while let Some(field) = multipart
            .next_field()
            .await
            .map_err(|_| AppError::InvalidMultipartContent)?
        {
            match field.name() {
                Some("text") => request.text = field.text().await.unwrap_or_default(),
                Some("media") => {
                    let file = File::from_field(field).await?;
                    request.files.push(file);
                }
                _ => (),
            }
        }
        Ok(request)
    }
}

#[async_trait]
impl ConstructableRequest for UpdatePostRequest {
    async fn parse<S: Send + Sync>(req: Request, state: &S) -> Result<Self, AppError>
    where
        Self: Sized,
    {
        let user_id = extract_user_id(&req)?;
        let post_id = extract_post_id(&req)?;
        let multipart = Multipart::from_request(req, &state)
            .await
            .map_err(|_| AppError::InvalidMultipartContent)?;

        UpdatePostRequest::from_multipart(multipart, user_id, post_id).await
    }
}

impl Validate for UpdatePostRequest {
    fn validate(&self) -> Result<(), AppError> {
        if self.text.trim().is_empty() {
            return Err(AppError::ValidationError(
                "Text field is required.".to_string(),
            ));
        }

        Ok(())
    }
}
