use axum::http::StatusCode;
use thiserror::Error;
use crate::errors::ProblemResponse;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Validation failed: {0}")]
    Failed(String),

    #[error("Invalid User ID")]
    InvalidUserId,

    #[error("Invalid Post ID")]
    InvalidPostId,

    #[error("Invalid URI: {0}")]
    InvalidUri(String),
}

impl ProblemResponse for ValidationError {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }

    fn title(&self) -> &str {
        "Validation Error"
    }

    fn detail(&self) -> String {
        self.to_string()
    }

    fn public_detail(&self) -> String {
        match self {
            ValidationError::Failed(err) => err.clone(),
            ValidationError::InvalidUserId => self.detail(),
            ValidationError::InvalidPostId => self.detail(),
            ValidationError::InvalidUri(err) => err.clone()
        }
    }
}