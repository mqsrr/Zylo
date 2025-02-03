use crate::errors::ProblemResponse;
use axum::http::StatusCode;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MongoError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] mongodb::error::Error),

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

    fn title(&self) -> &str {
        match self {
            MongoError::DatabaseError(_) => "Internal Server Error",
            MongoError::NotFound(_) => "Not Found",
        }
    }

    fn detail(&self) -> String {
        self.to_string()
    }

    fn public_detail(&self) -> String {
        match self {
            MongoError::NotFound(err) => err.clone(),
            _ => self.public_detail()
        }
    }
}