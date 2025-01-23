use axum::http::StatusCode;
use thiserror::Error;
use crate::errors::ProblemResponse;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Bearer token not found")]
    TokenNotFound,

    #[error("Invalid bearer token")]
    InvalidToken,
}

impl ProblemResponse for AuthError {
    fn status_code(&self) -> StatusCode {
        StatusCode::UNAUTHORIZED
    }

    fn title(&self) -> &'static str {
        "Authentication Error"
    }

    fn detail(&self) -> String {
        self.to_string()
    }
}