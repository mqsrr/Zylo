use crate::errors::app::ProblemResponse;
use axum::http::StatusCode;
use sqlx::Error;
use sqlx::migrate::MigrateError;
use thiserror::Error;
use tracing::log::warn;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Failed to create database pool: {0}")]
    PoolCreationError(String),
    #[error("Failed to run migrations: {0}")]
    MigrationError(#[from] MigrateError),
    #[error("SQLx Error: {0}")]
    SqlxError(#[source] sqlx::Error),
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    AlreadyExists(String),
}

impl ProblemResponse for DatabaseError {
    fn status_code(&self) -> StatusCode {
        match self {
            DatabaseError::PoolCreationError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            DatabaseError::MigrationError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            DatabaseError::SqlxError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            DatabaseError::NotFound(_) => StatusCode::NOT_FOUND,
            DatabaseError::AlreadyExists(_) => StatusCode::BAD_REQUEST,

        }
    }

    fn title(&self) -> &'static str {
        match self {
            DatabaseError::PoolCreationError(_) => "Internal Server Error",
            DatabaseError::MigrationError(_) => "Internal Server Error",
            DatabaseError::SqlxError(_) => "Internal Server Error",
            DatabaseError::NotFound(_) => "Not Found",
            DatabaseError::AlreadyExists(_) => "Bad Request",
        }
    }

    fn detail(&self) -> String {
        self.to_string()
    }

    fn public_detail(&self) -> &str {
        match self {
            DatabaseError::PoolCreationError(_) => "Internal Server Error",
            DatabaseError::MigrationError(_) => "Internal Server Error",
            DatabaseError::SqlxError(_) => "Internal Server Error",
            DatabaseError::NotFound(public_detail) => public_detail,
            DatabaseError::AlreadyExists(_) => "Resource with given id already exists",
        }
    }
}


impl From<sqlx::Error> for DatabaseError {
    fn from(value: Error) -> Self {
        match value {
            Error::Database(db_err) if db_err.is_foreign_key_violation() => {
                DatabaseError::NotFound("Post or user does not exist".into())
            }
            Error::Database(db_err) if db_err.is_unique_violation() => {
                warn!("{:?}", db_err);
                DatabaseError::AlreadyExists("Resource with given id already exists".into())
            }

            Error::RowNotFound => DatabaseError::NotFound("Resource not found".into()),
            _ => DatabaseError::SqlxError(value),
        }
    }
}