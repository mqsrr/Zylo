use axum::http::StatusCode;
use thiserror::Error;
use tonic::{Code, Status};
use tracing::log::warn;
use crate::errors::grpc::GrpcError::{BadRequest, Internal, NotFound};
use crate::errors::ProblemResponse;

#[derive(Error, Debug)]
pub enum GrpcError {
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    Internal(String)
}

impl From<Status> for GrpcError {
    fn from(value: Status) -> Self {
        let grpc_error = match value.code() {
            Code::Unknown | Code::InvalidArgument | Code::AlreadyExists => {
                BadRequest(value.message().into())
            }
            Code::NotFound => NotFound(value.message().into()),
            _ => Internal(value.message().into()),
        };

        match grpc_error {
            Internal(_) | BadRequest(_) => warn!("gRPC error occurred: {:?}", grpc_error),
            _ => {}
        }

        grpc_error
    }
}


impl ProblemResponse for GrpcError {
    fn status_code(&self) -> StatusCode {
        match self {
            NotFound(_) => StatusCode::NOT_FOUND,
            BadRequest(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR
        }
    }

    fn title(&self) -> &str {
        match self {
            NotFound(_) => "Not Found",
            BadRequest(_) => "Bad Request",
            _ => "Internal Server Error"
        }
    }

    fn detail(&self) -> String {
        match self {
            NotFound(err) => err.to_string(),
            BadRequest(err) => err.to_string(),
            Internal(err) => err.to_string(),
        }
    }

    fn public_detail(&self) -> &str {
        match self {
            NotFound(err) => err,
            BadRequest(err) => err,
            _ => "An unexpected server error occurred. Please try again later.",
        }
    }
}