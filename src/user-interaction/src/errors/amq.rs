use crate::errors::app::ProblemResponse;
use axum::http::StatusCode;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AmqError {
    #[error("Failed to connect to RabbitMQ server")]
    ConnectionError(#[from] lapin::Error),

    #[error("Failed to deserialize message: {0}")]
    DeserializeError(#[from] serde_json::Error),
}

impl ProblemResponse for AmqError {
    fn status_code(&self) -> StatusCode {
        match self {
            AmqError::ConnectionError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AmqError::DeserializeError(_) => StatusCode::BAD_REQUEST,
        }
    }

    fn title(&self) -> &'static str {
        match self {
            AmqError::ConnectionError(_) => "RabbitMQ Connection Error",
            AmqError::DeserializeError(_) => "Message Deserialization Error",
        }
    }

    fn detail(&self) -> String {
        self.to_string()
    }
}
