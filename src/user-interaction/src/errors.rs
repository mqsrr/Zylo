use std::env::VarError;
use aws_sdk_s3::operation::get_object::GetObjectError;
use aws_sdk_s3::operation::head_object::HeadObjectError;
use axum::extract::rejection::JsonRejection;
use axum::response::{IntoResponse, Response};
use axum::http::StatusCode;
use axum::Json;
use redis::RedisError;
use reqwest::Error;
use serde_json::json;
use thiserror::Error;
use tokio_cron_scheduler::JobSchedulerError;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Json Error: {0}")]
    InvalidJsonContent(#[from] JsonRejection),
    #[error("S3 Head object error: {0}")]
    S3HeadObjectError(#[from] HeadObjectError),   
    #[error("S3 Get object error: {0}")]
    S3GetObjectError(#[from] GetObjectError),
    #[error("Postgres Error: {0}")]
    PostgresDbErr(#[from] sqlx::Error),
    #[error("Redis Error: {0}")]
    RedisError(#[from] RedisError),
    #[error("Validation failed: {0}")]
    ValidationError(String),
    #[error("Bearer token not found")]
    BearerTokenNotFound,    
    #[error("User has already interacted with this action")]
    UserInteractionAlreadyAssigned,    
    #[error("User interaction has not been found")]
    UserInteractionNotFound,
    #[error("Bearer token is not valid")]
    InvalidBearerToken,    
    #[error("Schedule job error: {0}")]
    JobScheduleError(#[from] JobSchedulerError),
    #[error("Env error: {0}")]
    EnvironmentVariableNotFound(#[from] VarError),
    #[error("Request failed: {0}")]
    RequestError(#[from] Error),
    #[error("Error: {0}")]
    LapinError(#[from] lapin::Error),
    #[error("Error: {0}")]
    TonicStatus(#[from] tonic::Status)
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            AppError::InvalidJsonContent(_) => StatusCode::BAD_REQUEST,
            AppError::PostgresDbErr(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::RedisError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::JobScheduleError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::EnvironmentVariableNotFound(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::RequestError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::LapinError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::S3HeadObjectError(_) | AppError::S3GetObjectError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::TonicStatus(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::ValidationError(_) => StatusCode::BAD_REQUEST,
            AppError::BearerTokenNotFound | AppError::InvalidBearerToken => StatusCode::UNAUTHORIZED,
            AppError::UserInteractionAlreadyAssigned => StatusCode::OK,
            AppError::UserInteractionNotFound => StatusCode::NOT_FOUND,
        };
        (status, Json(json!({ "error": self.to_string() }))).into_response()
    }
}

pub trait Validate {
    fn validate(&self) -> Result<(), AppError>;
}