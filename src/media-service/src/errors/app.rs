use std::env::VarError;
use axum::extract::rejection::JsonRejection;
use crate::errors;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use opentelemetry::trace::TraceContextExt;
use serde_json::json;
use thiserror::Error;
use reqwest::Error as ReqwestError;
use tonic::Status;
use tracing::{error, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;

pub trait ProblemResponse {
    fn status_code(&self) -> StatusCode;
    fn title(&self) -> &str;
    fn detail(&self) -> String;

    fn public_detail(&self) -> String {
        String::from("An unexpected server error occurred. Please try again later.")
    }

    fn to_response(&self) -> Response {
        let context = Span::current().context();
        let trace_id = context.span().span_context().trace_id().to_string();
        let body = json!({
            "type": format!("https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/{}", self.status_code()),
            "title": self.title(),
            "status": self.status_code().as_u16(),
            "detail": self.public_detail(),
            "traceId": trace_id
        });

        (self.status_code(), Json(body)).into_response()
    }
}
#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    AmqError(#[from] errors::AmqError),
    #[error(transparent)]
    MongoError(#[from] errors::MongoError),
    #[error(transparent)]
    RedisError(#[from] errors::RedisError),
    #[error(transparent)]
    AuthError(#[from] errors::AuthError),
    #[error(transparent)]
    ValidationError(#[from] errors::ValidationError),
    #[error(transparent)]
    S3Error(#[from] errors::S3Error),
    
    #[error("Error making the request: {0}")]
    ReqwestError(#[from] ReqwestError),
    #[error("Json Error: {0}")]
    InvalidJsonContent(#[from] JsonRejection),
    #[error("Env error: {0}")]
    EnvironmentVariableNotFound(#[from] VarError),
    #[error("{0}")]
    BadRequest(String),       
    #[error("{0}")]
    NotFound(String),
}

impl ProblemResponse for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::RedisError(err) => err.status_code(),
            AppError::MongoError(err) => err.status_code(),
            AppError::AmqError(err) => err.status_code(),
            AppError::ValidationError(err) => err.status_code(),
            AppError::AuthError(err) => err.status_code(),
            AppError::S3Error(err) => err.status_code(),
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn title(&self) -> &str {
        match self {
            AppError::ValidationError(err) => err.title(),
            AppError::AuthError(err) => err.title(),
            AppError::RedisError(err) => err.title(),
            AppError::S3Error(err) => err.title(),
            AppError::MongoError(err) => err.title(),
            AppError::AmqError(err) => err.title(),
            
            AppError::NotFound(_) => "Resource Not Found",
            AppError::BadRequest(_) => "Bad Request",
            _ => "Internal Server Error"
        }
    }

    fn detail(&self) -> String {
        self.to_string()
    }

    fn public_detail(&self) -> String {
        match self {
            AppError::ValidationError(err) => err.public_detail(),
            AppError::AuthError(err) => err.public_detail(),
            AppError::RedisError(err) => err.public_detail(),
            AppError::MongoError(err) => err.public_detail(),
            AppError::AmqError(err) => err.public_detail(),
            AppError::S3Error(err) => err.public_detail(),
            AppError::BadRequest(err) => err.clone(),
            AppError::NotFound(err) => err.clone(),

            _ => String::from("An unexpected server error occurred. Please try again later.")
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        if self.status_code() == StatusCode::INTERNAL_SERVER_ERROR {
            error!("Internal error: {}", self.detail());
        }

        self.to_response()
    }
}

impl From<AppError> for Status {
    fn from(value: AppError) -> Self {
        match value.status_code() {             
            StatusCode::NOT_FOUND => Status::not_found(value.public_detail()),
            _ => Status::internal(value.public_detail()), } 
    }
}