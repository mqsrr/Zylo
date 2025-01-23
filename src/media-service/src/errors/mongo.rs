use crate::errors::ProblemResponse;
use axum::http::StatusCode;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MongoError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("{0}")]
    NotFound(String),
    
}

impl ProblemResponse for MongoError {
    fn status_code(&self) -> StatusCode {
        match self {
            MongoError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            MongoError::NotFound(_) => StatusCode::NOT_FOUND,
        }
    }

    fn title(&self) -> &'static str {
        match self {
            MongoError::DatabaseError(_) => "Database Error",
            MongoError::NotFound(_) => "Resource Not Found",
        }
    }

    fn detail(&self) -> String {
        self.to_string()
    }

    fn public_detail(&self) -> &str {
        match self {
            MongoError::NotFound(err) => err,
            _ => "An unexpected server error occurred. Please try again later."
        }
    }
}