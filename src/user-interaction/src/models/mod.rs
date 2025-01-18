use async_trait::async_trait;
use crate::errors;

pub mod reply;
pub mod app_state;
pub mod amq_message;

#[async_trait]
pub trait Finalizer {
    async fn finalize(&self) -> Result<(), errors::AppError>;
}