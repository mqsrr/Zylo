use crate::errors;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use std::env::VarError;
use opentelemetry::trace::TraceContextExt;
use thiserror::Error;
use tracing::{error, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use reqwest::Error as ReqwestError;

pub trait ProblemResponse {
    fn status_code(&self) -> StatusCode;
    fn title(&self) -> &str;
    fn detail(&self) -> String;

    fn public_detail(&self) -> &str {
        "An unexpected server error occurred. Please try again later."
    }

    fn to_response(&self) -> Response {
        let context = Span::current().context();
        let trace_id = context.span().span_context().trace_id().to_string();
        
        let body = json!({
            "type": format!("https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/{}", self.status_code()),
            "title": self.title(),
            "status": self.status_code().as_u16(),
            "detail": self.public_detail(),
            "traceId": trace_id
        });

        (self.status_code(), Json(body)).into_response()
    }
}
#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    Auth(#[from] errors::AuthError),
    #[error(transparent)]
    Grpc(#[from] errors::GrpcError),

    #[error("Error making the request: {0}")]
    ReqwestError(#[from] ReqwestError),
    #[error("Json Error: {0}")]
    InvalidJsonContent(#[from] JsonRejection),
    #[error("Env error: {0}")]
    EnvironmentVariableNotFound(#[from] VarError),
}

impl ProblemResponse for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::Auth(err) => err.status_code(),
            AppError::Grpc(err) => err.status_code(),
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn title(&self) -> &str {
        match self {
            AppError::Auth(err) => err.title(),
            AppError::Grpc(err) => err.title(),

            _ => "Internal Server Error"
        }
    }

    fn detail(&self) -> String {
        self.to_string()
    }

    fn public_detail(&self) -> &str {
        match self {
            AppError::Grpc(err) => err.public_detail(),
            AppError::Auth(err) => err.public_detail(),
            
            _ => "An unexpected server error occurred. Please try again later."
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        if self.status_code().is_server_error(){
            error!("Internal error: {}", self.detail());
        }

        self.to_response()
    }
}
