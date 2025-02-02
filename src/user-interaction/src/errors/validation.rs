use axum::http::StatusCode;
use thiserror::Error;
use crate::errors::ProblemResponse;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Validation failed: {0}")]
    Failed(String),

    #[error("Invalid User ID")]
    InvalidUserId,
    
    #[error("Invalid replied object ID")]
    InvalidReplyToId,
}

impl ProblemResponse for ValidationError {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }

    fn title(&self) -> &'static str {
        "Validation Error"
    }

    fn detail(&self) -> String {
        self.to_string()
    }
}