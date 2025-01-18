use crate::errors;
use crate::errors::app::ProblemResponse;
use axum::http::StatusCode;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RedisError {
    #[error("Could not connect to redis server")]
    ConnectionError,
    #[error("Background worker for redis back up could not be started: {0}")]
    BackgroundWorkerStartupError(String),

    #[error("Redis operation failed: {operation}, key: {key}, error: {source}")]
    OperationError {
        operation: String,
        key: String,
        #[source]
        source: redis::RedisError,
    },
    #[error(transparent)]
    SerializationError(#[from] serde_json::Error),
}

impl ProblemResponse for RedisError {
    fn status_code(&self) -> StatusCode {
        match self {
            RedisError::ConnectionError => StatusCode::INTERNAL_SERVER_ERROR,
            RedisError::BackgroundWorkerStartupError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            RedisError::OperationError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            RedisError::SerializationError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn title(&self) -> &'static str {
        match self {
            RedisError::ConnectionError => "Internal Server Error",
            RedisError::BackgroundWorkerStartupError(_) => "Internal Server Error",
            RedisError::OperationError { .. } => "Internal Server Error",
            RedisError::SerializationError(_) => "Internal Server Error",
        }
    }

    fn detail(&self) -> String {
        self.to_string()
    }
}

pub fn redis_op_error(operation: &str, key: &str, source: redis::RedisError) -> errors::RedisError {
    RedisError::OperationError {
        operation: operation.to_string(),
        key: key.to_string(),
        source,
    }
}
