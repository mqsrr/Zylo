use std::collections::HashMap;
use async_trait::async_trait;
use prometheus::{HistogramVec, IntCounterVec, Opts, Registry};
use tracing::{field, info_span};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use ulid::Ulid;
use crate::errors;
use crate::errors::ProblemResponse;
use crate::repositories::interaction_repo::InteractionRepository;

#[derive(Clone)]
pub struct ObservableInteractionRepository<I: InteractionRepository> {
    inner: I,
    request_count: IntCounterVec,
    request_latency: HistogramVec,
}
impl<I: InteractionRepository> ObservableInteractionRepository<I> {
    pub fn new(inner: I, registry: &Registry) -> Result<Self, errors::AppError> {
        const LATENCY_BUCKETS: &[f64] = &[
            0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
        ];

        let request_count = IntCounterVec::new(
            Opts::new(
                "interaction_repo_requests_total",
                "Total requests to InteractionRepository",
            ),
            &["method", "operation", "status"],
        )
            .map_err(|e| errors::ObservabilityError::MetricRegistrationError(e.to_string()))?;

        let request_latency = HistogramVec::new(
            prometheus::HistogramOpts::new(
                "interaction_repo_request_duration_seconds",
                "Latency of InteractionRepository methods",
            )
                .buckets(LATENCY_BUCKETS.to_vec()),
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

    async fn track_method<T, F, E: ProblemResponse + ToString>(
        &self,
        method_name: &str,
        operation_name: &str,
        target: &str,
        operation: F,
    ) -> Result<T, E>
    where
        F: std::future::Future<Output = Result<T, E>>,
    {
        let start_time = tokio::time::Instant::now();
        let span = info_span!(
            "",
            "otel.name" = operation_name,
            "db.system" = "cache",
            "db.target" = target,
            "method.name" = method_name,
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
                    span.record("error.type", "redis_error")
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
impl<R: InteractionRepository> InteractionRepository for ObservableInteractionRepository<R> {
    async fn get_user_interactions(
        &self,
        user_id: &str,
        fields: Vec<String>,
    ) -> Result<HashMap<String, bool>, errors::RedisError> {
        self.track_method(
            "get_user_interactions",
            "SISMEMBER",
            "user-interaction:posts:{ids}:likes",
            self.inner.get_user_interactions(user_id, fields),
        )
            .await
    }

    async fn is_user_liked(&self, user_id: &str, field: &str) -> Result<bool, errors::RedisError> {
        self.track_method(
            "is_user_liked",
            "SISMEMBER",
            "user-interaction:posts:{id}:likes",
            self.inner.is_user_liked(user_id, field),
        )
            .await
    }

    async fn like_post(
        &self,
        user_id: String,
        post_id: String,
    ) -> Result<bool, errors::RedisError> {
        self.track_method(
            "like_post",
            "SADD",
            "user-interaction:posts:{id}:likes",
            self.inner.like_post(user_id, post_id),
        )
            .await
    }

    async fn unlike_post(
        &self,
        user_id: String,
        post_id: String,
    ) -> Result<bool, errors::RedisError> {
        self.track_method(
            "unlike_post",
            "SREM",
            "user-interaction:posts:{id}:likes",
            self.inner.unlike_post(user_id, post_id),
        )
            .await
    }

    async fn get_interactions(
        &self,
        key: &str,
        fields: &Vec<String>,
    ) -> Result<HashMap<Ulid, i32>, errors::RedisError> {
        self.track_method(
            "get_interactions",
            "HMGET",
            key,
            self.inner.get_interactions(key, fields),
        )
            .await
    }

    async fn get_interaction(&self, key: &str, field: &str) -> Result<i32, errors::RedisError> {
        self.track_method(
            "get_interaction",
            "HGET",
            key,
            self.inner.get_interaction(key, field),
        )
            .await
    }

    async fn add_view(&self, user_id: String, post_id: String) -> Result<bool, errors::RedisError> {
        self.track_method(
            "add_view",
            "SADD",
            "user-interaction:posts:{id}:views",
            self.inner.add_view(user_id, post_id),
        )
            .await
    }

    async fn delete_interactions(&self, post_id: &str) -> Result<(), errors::RedisError> {
        self.track_method(
            "delete_interactions",
            "HDEL | SREM",
            "user-interaction:posts:{id}:{likes/views} | user-interaction:replies",
            self.inner.delete_interactions(post_id),
        )
            .await
    }

    async fn delete_many_interactions(
        &self,
        posts_ids: &Vec<String>,
    ) -> Result<(), errors::RedisError> {
        self.track_method(
            "delete_many_interactions",
            "PIPELINE_HDEL | SREM",
            "user-interaction:posts:{id}:{likes/views} | user-interaction:replies",
            self.inner.delete_many_interactions(posts_ids),
        )
            .await
    }
}
