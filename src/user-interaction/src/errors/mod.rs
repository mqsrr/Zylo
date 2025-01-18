mod amq;
mod app;
mod database;
mod redis;

pub use amq::*;
pub use app::AppError;
pub use app::ProblemResponse;
pub use database::DatabaseError;
pub use redis::{redis_op_error, RedisError};
use thiserror::Error;
#[derive(Debug, Error)]
pub enum ObservabilityError {
    #[error("Failed to register Prometheus metric: {0}")]
    MetricRegistrationError(String),
}
