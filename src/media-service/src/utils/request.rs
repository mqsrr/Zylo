use crate::models::file::File;
use async_trait::async_trait;
use axum::extract::{FromRequest, Multipart, Request};
use serde::{Deserialize, Serialize};
use tracing::log::warn;
use ulid::Ulid;
use crate::errors;

#[async_trait]
pub trait ConstructableRequest {
    async fn parse<S: Send + Sync>(request: Request, state: &S) -> Result<Self, errors::AppError>
    where
        Self: Sized;
}
pub trait Validate {
    fn validate(&self) -> Result<(), errors::ValidationError>;
}

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    #[serde(rename = "next")]
    pub next: Option<Ulid>,
    #[serde(rename = "perPage")]
    pub per_page: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    #[serde(rename = "perPage")]
    pub per_page: u32,
    #[serde(rename = "hasNextPage")]
    pub has_next_page: bool,
    #[serde(rename = "next")]
    pub next_cursor: String,
}

impl<T> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, per_page: u32, has_next_page: bool, next_cursor: String) -> Self {
        Self {
            data,
            per_page,
            has_next_page,
            next_cursor,
        }
    }
    pub fn from_page<F, U>(paginated_response: PaginatedResponse<T>, f: F) -> PaginatedResponse<U>
    where
        F: Fn(T) -> U
    {
        PaginatedResponse::<U> {
            data: paginated_response.data.into_iter().map(f).collect(),
            per_page: paginated_response.per_page,
            has_next_page: paginated_response.has_next_page,
            next_cursor: paginated_response.next_cursor,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreatePostRequest {
    pub user_id: Ulid,
    pub text: String,
    pub files: Vec<File>,
}

impl CreatePostRequest {
    pub fn new(user_id: Ulid) -> Self {
        Self {
            user_id,
            text: String::new(),
            files: Vec::new(),
        }
    }

    pub async fn from_multipart(mut multipart: Multipart, user_id: Ulid) -> Result<Self, errors::AppError> {
        let mut request = Self::new(user_id);

        while let Some(field) = multipart
            .next_field()
            .await
            .map_err(|_| errors::AppError::BadRequest("Invalid multipart data".to_string()))?
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

impl Validate for CreatePostRequest {
    fn validate(&self) -> Result<(), errors::ValidationError> {
   
        if self.user_id.is_nil() {
            return Err(errors::ValidationError::Failed("User id cannot be empty".to_string()));
        }
        
        if self.text.trim().is_empty() {
            return Err(errors::ValidationError::Failed("Post context cannot be empty".to_string()));
        }
        
        Ok(())
    }
}

impl Validate for PaginationParams {
    fn validate(&self) -> Result<(), errors::ValidationError> {
        if let Some(per_page) = self.per_page {
            if per_page < 1 {
                return Err(errors::ValidationError::Failed("perPage cannot be less than 1".to_string()));
            }
            
        }
        
        Ok(())
    }
}

impl Validate for UpdatePostRequest {
    fn validate(&self) -> Result<(), errors::ValidationError> {

        if self.id.is_nil() {
            return Err(errors::ValidationError::Failed("Post id cannot be empty".to_string()));
        }

        if self.text.trim().is_empty() {
            return Err(errors::ValidationError::Failed("Post context cannot be empty".to_string()));
        }

        Ok(())
    }
}

#[async_trait]
impl ConstructableRequest for CreatePostRequest {
    async fn parse<S: Send + Sync>(req: Request, state: &S) -> Result<Self, errors::AppError>
    where
        Self: Sized,
    {
        let user_id = extract_user_id(&req)?;
        let multipart = Multipart::from_request(req, &state)
            .await
            .map_err(|_| errors::ValidationError::InvalidUserId)?;

        CreatePostRequest::from_multipart(multipart, user_id).await
    }
}
fn extract_user_id(req: &Request) -> Result<Ulid, errors::AppError> {
    let uri_path = req.uri().path();
    let user_id_str = uri_path
        .split('/')
        .nth(3)
        .ok_or_else(|| errors::ValidationError::InvalidUri("Could not find user id".to_string()))?;

    Ulid::from_string(user_id_str).map_err(|_| errors::ValidationError::InvalidUserId.into())
}

fn extract_post_id(req: &Request) -> Result<Ulid, errors::AppError> {
    let uri_path = req.uri().path();
    let post_id_str = uri_path
        .split('/')
        .nth(5)
        .ok_or_else(|| errors::ValidationError::InvalidUri("Could not find post id".to_string()))?;

    Ulid::from_string(post_id_str).map_err(|_| errors::ValidationError::InvalidPostId.into())
}

#[derive(Debug, Clone)]
pub struct UpdatePostRequest {
    pub id: Ulid,
    pub text: String,
    pub files: Vec<File>,
}

impl UpdatePostRequest {
    pub async fn from_multipart(
        mut multipart: Multipart,
        post_id: Ulid,
    ) -> Result<Self, errors::AppError> {
        let mut request = Self {
            id: post_id,
            text: String::new(),
            files: Vec::new(),
        };

        while let Some(field) = multipart
            .next_field()
            .await
            .map_err(|_| errors::ValidationError::Failed("Invalid multipart data".to_string()))?
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
    async fn parse<S: Send + Sync>(req: Request, state: &S) -> Result<Self, errors::AppError>
    where
        Self: Sized,
    {
        let post_id = extract_post_id(&req)?;
        let multipart = Multipart::from_request(req, &state)
            .await
            .map_err(|_| errors::ValidationError::Failed("Invalid multipart data".to_string()))?;

        UpdatePostRequest::from_multipart(multipart, post_id).await
    }
}