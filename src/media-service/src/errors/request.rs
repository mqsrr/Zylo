use crate::errors::ProblemResponse;
use axum::http::StatusCode;
use thiserror::Error;
use tonic::Status;

#[derive(Error, Debug)]
pub enum RequestError {
    #[error("gRPC Error: {0}")]
    Grpc(#[from] Status),
}

impl ProblemResponse for RequestError {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    fn title(&self) -> &'static str {
        "Internal Server Error"
    }

    fn detail(&self) -> String {
        self.to_string()
    }
}