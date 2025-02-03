use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use crate::errors;

pub mod cache_service_decorator;
pub mod grpc_server_decorator;
pub mod user_repo_decorator;
pub mod post_repo_decorator;
pub mod s3_service_decorator;

fn trace_server_error<T, E>(result: Result<T, E>, span: &Span, error_type: &str) -> Result<T, E>
where
    E: errors::ProblemResponse + ToString,
{
    result.inspect_err(|err| {
        if err.status_code().is_server_error() {
            span.record("error.type", error_type)
                .set_status(opentelemetry::trace::Status::error(err.to_string()));
        }
    })
}