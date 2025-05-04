use axum::http::StatusCode;
use thiserror::Error;
use crate::errors::ProblemResponse;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Bearer token not found")]
    TokenNotFound,

    #[error("Invalid bearer token")]
    InvalidToken,

    #[error("Email is not confirmed")]
    UnverifiedEmail,
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

    fn public_detail(&self) -> String {
        match self {
            AuthError::TokenNotFound => String::from("Bearer token not found"),
            AuthError::InvalidToken => String::from("Invalid token"),
            AuthError::UnverifiedEmail => String::from("Email is not confirmed"),
        }
    }
}