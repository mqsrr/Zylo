use aws_sdk_s3::operation::delete_object::DeleteObjectError;
use aws_sdk_s3::operation::get_object::GetObjectError;
use aws_sdk_s3::operation::head_object::HeadObjectError;
use aws_sdk_s3::operation::put_object::PutObjectError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use redis::RedisError;
use reqwest::Error;
use serde_json::json;
use std::env::VarError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Invalid Multipart Content")]
    InvalidMultipartContent,
    #[error("Invalid URI: {0}")]
    InvalidUri(String),
    #[error("Invalid User ID")]
    InvalidUserId,

    #[error("Env error: {0}")]
    EnvironmentVariableNotFound(#[from] VarError),
    #[error("Invalid Post ID")]
    InvalidPostId,
    #[error("MongoDB Error: {0}")]
    MongoDbError(#[from] mongodb::error::Error),
    #[error("File could not be uploaded: {0}")]
    S3UploadFileError(#[from] PutObjectError),
    #[error("File could not be deleted: {0}")]
    S3DeleteFileError(#[from] DeleteObjectError),
    #[error("File could not be retrieved: {0}")]
    S3GetObjectFileError(#[from] GetObjectError),
    #[error("Could not retrieve metadata of object: {0}")]
    S3HeadObjectFileError(#[from] HeadObjectError),
    #[error("Request failed: {0}")]
    RequestError(#[from] Error),
    #[error("Error: {0}")]
    LapinError(#[from] lapin::Error),
    #[error("Error: {0}")]
    TonicStatus(#[from] tonic::Status),
    #[error("Validation failed: {0}")]
    ValidationError(String),
    #[error("{0}")]
    NotFound(String),
    #[error("Redis Error: {0}")]
    RedisError(#[from] RedisError),
    #[error("Bearer token not found")]
    BearerTokenNotFound,
    #[error("Bearer is not valid")]
    InvalidBearerToken,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            AppError::InvalidMultipartContent => StatusCode::BAD_REQUEST,
            AppError::InvalidUri(_) | AppError::InvalidUserId | AppError::InvalidPostId => {
                StatusCode::BAD_REQUEST
            }
            AppError::MongoDbError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::S3GetObjectFileError(_)
            | AppError::S3UploadFileError(_)
            | AppError::S3DeleteFileError(_)
            | AppError::S3HeadObjectFileError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::RedisError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::RequestError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::EnvironmentVariableNotFound(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::LapinError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::ValidationError(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::BearerTokenNotFound | AppError::InvalidBearerToken => {
                StatusCode::UNAUTHORIZED
            }
            AppError::TonicStatus(_) => StatusCode::INTERNAL_SERVER_ERROR
        };
        (status, Json(json!({ "error": self.to_string() }))).into_response()
    }
}

pub trait Validate {
    fn validate(&self) -> Result<(), AppError>;
}
