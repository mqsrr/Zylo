use axum::http::StatusCode;
use thiserror::Error;
use crate::errors::{ProblemResponse};

#[derive(Debug, Error)]
pub enum ObservabilityError {
    #[error("Failed to register Prometheus metric: {0}")]
    MetricRegistration(String),
}

impl ProblemResponse for ObservabilityError {
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