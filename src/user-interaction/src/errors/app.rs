use crate::errors;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use reqwest::Error;
use serde_json::json;
use std::env::VarError;
use opentelemetry::trace::TraceContextExt;
use thiserror::Error;
use tracing::{error, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;

pub trait ProblemResponse {
    fn status_code(&self) -> StatusCode;
    fn title(&self) -> &str;
    fn detail(&self) -> String;

    fn public_detail(&self) -> &str {
        "An unexpected server error occurred. Please try again later."
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
    #[error("Json Error: {0}")]
    InvalidJsonContent(#[from] JsonRejection),
    #[error("Validation failed: {0}")]
    ValidationError(String),
    #[error("Bearer token not found")]
    BearerTokenNotFound,
    #[error("Bearer token is not valid")]
    InvalidBearerToken,
    #[error("Env error: {0}")]
    EnvironmentVariableNotFound(#[from] VarError),

    #[error("Error making the request: {0}")]
    ReqwestError(#[from] Error),
    #[error(transparent)]
    RedisError(#[from] errors::RedisError),
    #[error(transparent)]
    DatabaseError(#[from] errors::DatabaseError),
    #[error(transparent)]
    AmqError(#[from] errors::AmqError),
    #[error(transparent)]
    ObservabilityError(#[from] errors::ObservabilityError),
}

impl ProblemResponse for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::InvalidJsonContent(_) | AppError::ValidationError(_) => {
                StatusCode::BAD_REQUEST
            }
            AppError::RedisError(err) => err.status_code(),
            AppError::DatabaseError(err) => err.status_code(),
            AppError::AmqError(err) => err.status_code(),
            AppError::EnvironmentVariableNotFound(_)
            | AppError::ReqwestError(_)
            | AppError::ObservabilityError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::BearerTokenNotFound | AppError::InvalidBearerToken => {
                StatusCode::UNAUTHORIZED
            }
        }
    }

    fn title(&self) -> &str {
        match self {
            AppError::InvalidJsonContent(_) => "Bad Request",
            AppError::ValidationError(_) => "Bad Request",
            AppError::BearerTokenNotFound => "Unauthorized",
            AppError::InvalidBearerToken => "Unauthorized",
            AppError::RedisError(err) => err.title(),
            AppError::DatabaseError(err) => err.title(),
            AppError::AmqError(err) => err.title(),
            AppError::ObservabilityError(_) => "Internal Server Error",
            AppError::EnvironmentVariableNotFound(_) => "Internal Server Error",
            AppError::ReqwestError(_) => "Internal Server Error",
        }
    }

    fn detail(&self) -> String {
        self.to_string()
    }

    fn public_detail(&self) -> &str {
        match self {
            AppError::BearerTokenNotFound => "Authorization token is missing. Please provide a valid token.",
            AppError::InvalidBearerToken => "The provided authorization token is invalid. Please check your credentials.",

            AppError::InvalidJsonContent(_) => "The provided JSON content is invalid. Please ensure the request body is properly formatted.",
            AppError::ValidationError(_) => "The request contains invalid data. Please verify your input and try again.",
            AppError::ReqwestError(_) => "The request could not be processed. Please check the parameters and try again.",

            AppError::RedisError(err) => err.public_detail(),
            AppError::DatabaseError(err) => err.public_detail(),
            AppError::AmqError(err) => err.public_detail(),
            AppError::ObservabilityError(_) => {
                "An unexpected server error occurred. Please try again later."
            }
            AppError::EnvironmentVariableNotFound(_) => {
                "A server error occurred due to a missing configuration setting. Please contact support."
            }

            _ => "An unexpected server error occurred. Please try again later."
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
