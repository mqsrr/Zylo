use crate::errors::app::ProblemResponse;
use axum::http::StatusCode;
use sqlx::migrate::MigrateError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Failed to create database pool: {0}")]
    PoolCreationError(String),
    #[error("Failed to run migrations: {0}")]
    MigrationError(#[from] MigrateError),
    #[error("SQLx Error: {0}")]
    SqlxError(#[from] sqlx::Error),
    #[error("Resource not found: {0}")]
    NotFound(String),
}

impl ProblemResponse for DatabaseError {
    fn status_code(&self) -> StatusCode {
        match self {
            DatabaseError::PoolCreationError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            DatabaseError::MigrationError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            DatabaseError::SqlxError(_) => StatusCode::BAD_REQUEST,
            DatabaseError::NotFound(_) => StatusCode::NOT_FOUND
        }
    }

    fn title(&self) -> &'static str {
        match self {
            DatabaseError::PoolCreationError(_) => "Internal Server Error",
            DatabaseError::MigrationError(_) => "Internal Server Error",
            DatabaseError::SqlxError(_) => "Internal Server Error",
            DatabaseError::NotFound(_) => "Not Found"
        }
    }

    fn public_detail(&self) -> &str {
        match self {
            DatabaseError::PoolCreationError(_) => "Internal Server Error",
            DatabaseError::MigrationError(_) => "Internal Server Error",
            DatabaseError::SqlxError(_) => "Internal Server Error",
            DatabaseError::NotFound(public_detail) => public_detail
        }
    }

    fn detail(&self) -> String {
        self.to_string()
    }
}
