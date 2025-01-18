use std::collections::HashMap;
use async_trait::async_trait;
use prometheus::{HistogramVec, IntCounterVec, Opts, Registry};
use tokio::time::Instant;
use tracing::{field, info_span};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use ulid::Ulid;
use crate::errors;
use crate::models::Finalizer;
use crate::models::reply::Reply;
use crate::repositories::reply_repo::ReplyRepository;

#[derive(Clone)]
pub struct ObservableReplyRepository<R: ReplyRepository> {
    inner: R,
    request_count: IntCounterVec,
    request_latency: HistogramVec,
}

impl<R: ReplyRepository> ObservableReplyRepository<R> {
    pub fn new(inner: R, registry: &Registry) -> Result<Self, errors::AppError> {
        const EXPONENTIAL_SECONDS: &[f64] = &[
            0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
        ];
        let request_count = IntCounterVec::new(
            Opts::new(
                "reply_repo_requests_total",
                "Total requests to ReplyRepository",
            ),
            &["method", "operation", "status"],
        )
            .map_err(|e| errors::ObservabilityError::MetricRegistrationError(e.to_string()))?;

        let request_latency = HistogramVec::new(
            prometheus::HistogramOpts::new(
                "reply_repo_request_duration_seconds",
                "Latency of ReplyRepository methods",
            )
                .buckets(EXPONENTIAL_SECONDS.to_vec()),
            &["method", "operation"],
        )
            .map_err(|e| errors::ObservabilityError::MetricRegistrationError(e.to_string()))?;

        registry
            .register(Box::new(request_count.clone()))
            .map_err(|e| errors::ObservabilityError::MetricRegistrationError(e.to_string()))?;
        registry
            .register(Box::new(request_latency.clone()))
            .map_err(|e| errors::ObservabilityError::MetricRegistrationError(e.to_string()))?;

        Ok(Self {
            inner,
            request_count,
            request_latency,
        })
    }

    async fn track_method<T, F, E: errors::ProblemResponse + ToString>(
        &self,
        method_name: &str,
        query_summary: &str,
        operation_name: &str,
        target: &str,
        operation: F,
    ) -> Result<T, E>
    where
        F: std::future::Future<Output = Result<T, E>>,
    {
        let start_time = Instant::now();
        let span = info_span!(
            "",
            "otel.name" = query_summary,
            "db.system" = "postgresql",
            "db.operation" = operation_name,
            "db.target" = target,
            "method.name" = method_name,
            "error.message" = field::Empty,
            "error.type" = field::Empty,
        );

        let result = operation.await;

        let status = if result.is_ok() { "success" } else { "error" };
        self.request_count
            .with_label_values(&[method_name, operation_name, status])
            .inc();

        self.request_latency
            .with_label_values(&[method_name, operation_name])
            .observe(start_time.elapsed().as_secs_f64());

        if let Err(ref err) = result {
            match err.status_code() {
                axum::http::StatusCode::INTERNAL_SERVER_ERROR => {
                    span.record("error.type", "database_error")
                        .set_status(opentelemetry::trace::Status::error(err.to_string()));
                    return result;
                }
                _ => {}
            }
        }

        span.set_status(opentelemetry::trace::Status::Ok);
        result
    }
}

#[async_trait]
impl<R: ReplyRepository + 'static> Finalizer for ObservableReplyRepository<R> {
    async fn finalize(&self) -> Result<(), errors::AppError> {
        self.inner.finalize().await
    }
}

#[async_trait]
impl<R: ReplyRepository + 'static> ReplyRepository for ObservableReplyRepository<R> {
    async fn get_all_from_post(&self, post_id: &Ulid) -> Result<Vec<Reply>, errors::DatabaseError> {
        self.track_method(
            "get_all_from_post",
            "SELECT replies by root_id",
            "SELECT",
            "replies",
            self.inner.get_all_from_post(post_id),
        )
            .await
    }

    async fn get_all_from_posts(
        &self,
        post_ids: &[Ulid],
    ) -> Result<HashMap<Ulid, Vec<Reply>>, errors::DatabaseError> {
        self.track_method(
            "get_all_from_posts",
            "SELECT replies by multiple root_ids",
            "SELECT",
            "replies",
            self.inner.get_all_from_posts(post_ids),
        )
            .await
    }

    async fn get_reply_path(&self, id: &Ulid) -> Result<String, errors::DatabaseError> {
        self.track_method(
            "get_reply_path",
            "SELECT reply path by id",
            "SELECT",
            "replies",
            self.inner.get_reply_path(id),
        )
            .await
    }

    async fn get_with_nested_by_path_prefix(
        &self,
        prefix: &str,
    ) -> Result<Vec<Reply>, errors::DatabaseError> {
        self.track_method(
            "get_with_nested_by_path_prefix",
            "SELECT replies with nested paths",
            "SELECT",
            "replies",
            self.inner.get_with_nested_by_path_prefix(prefix),
        )
            .await
    }

    async fn get_with_nested(&self, id: &Ulid) -> Result<Vec<Reply>, errors::DatabaseError> {
        self.track_method(
            "get_with_nested",
            "SELECT nested replies by id",
            "SELECT",
            "replies",
            self.inner.get_with_nested(id),
        )
            .await
    }

    async fn create(&self, reply: &Reply) -> Result<(), errors::DatabaseError> {
        self.track_method(
            "create",
            "INSERT reply",
            "INSERT",
            "replies",
            self.inner.create(reply),
        )
            .await
    }

    async fn update(&self, id: &Ulid, content: &str) -> Result<Reply, errors::DatabaseError> {
        self.track_method(
            "update",
            "UPDATE reply content",
            "UPDATE",
            "replies",
            self.inner.update(id, content),
        )
            .await
    }

    async fn delete(&self, id: &Ulid) -> Result<(), errors::DatabaseError> {
        self.track_method(
            "delete",
            "DELETE reply by id",
            "DELETE",
            "replies",
            self.inner.delete(id),
        )
            .await
    }

    async fn delete_all_by_user_id(&self, id: &Ulid) -> Result<Vec<String>, errors::DatabaseError> {
        self.track_method(
            "delete_all_by_user_id",
            "DELETE replies by user_id",
            "DELETE",
            "replies",
            self.inner.delete_all_by_user_id(id),
        )
            .await
    }

    async fn delete_all_by_post_id(&self, post_id: &Ulid) -> Result<(), errors::DatabaseError> {
        self.track_method(
            "delete_all_by_post_id",
            "DELETE replies by post_id",
            "DELETE",
            "replies",
            self.inner.delete_all_by_post_id(post_id),
        )
            .await
    }
}
