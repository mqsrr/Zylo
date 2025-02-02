use crate::errors::app::ProblemResponse;
use axum::http::StatusCode;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RedisError {
    #[error("Redis operation failed: {operation}, key: {key}, error: {source}")]
    Operation {
        operation: String,
        key: String,
        #[source]
        source: redis::RedisError,
    },
    #[error(transparent)]
    Serialization(#[from] serde_json::Error),
}

impl ProblemResponse for RedisError {
    fn status_code(&self) -> StatusCode {
        match self {
            _ => StatusCode::INTERNAL_SERVER_ERROR
        }
    }

    fn title(&self) -> &'static str {
        match self {
            _ => "Internal Server Error"
        }
    }

    fn detail(&self) -> String {
        self.to_string()
    }
}

pub fn redis_op_error(operation: &str, key: &str, source: redis::RedisError) -> RedisError {
    RedisError::Operation {
        operation: operation.to_string(),
        key: key.to_string(),
        source,
    }
}