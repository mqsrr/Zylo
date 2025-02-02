use crate::errors;
use crate::errors::DatabaseError;
use crate::repositories::posts_repo::{PostgresPostsRepository, PostsRepository};
use async_trait::async_trait;
use opentelemetry::metrics::{Counter, Histogram};
use opentelemetry::trace::SpanKind;
use opentelemetry::{global, KeyValue};
use sqlx::{PgPool};
use tokio::time::Instant;
use tracing::{field, info_span};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use ulid::Ulid;

pub struct DecoratedPostsRepository<P: PostsRepository> {
    posts_repo: P,
}

impl DecoratedPostsRepository<PostgresPostsRepository> {
    pub fn new(db: PgPool) -> Self {
        Self {
            posts_repo: PostgresPostsRepository::new(db),
        }
    }
}

impl<P: PostsRepository + 'static> DecoratedPostsRepository<P> {
    pub fn observable(self) -> DecoratedPostsRepository<ObservablePostsRepository<P>> {
        DecoratedPostsRepository {
            posts_repo: ObservablePostsRepository::new(self.posts_repo),
        }
    }

    pub fn build(self) -> P {
        self.posts_repo
    }
}

#[derive(Clone)]
pub struct ObservablePostsRepository<P: PostsRepository> {
    inner: P,
    request_count: Counter<u64>,
    request_latency: Histogram<f64>,
}

impl<P: PostsRepository> ObservablePostsRepository<P> {
    pub fn new(inner: P) -> Self {
        let boundaries = vec![
            0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
        ];

        let meter_provider = global::meter("user-interaction");
        let request_count = meter_provider
            .u64_counter("posts_repo_requests_total")
            .with_description("Total requests to PostsRepository")
            .with_unit("PostsRepository")
            .build();

        let request_latency = meter_provider
            .f64_histogram("posts_repo_request_duration_seconds")
            .with_description("Latency of PostsRepository methods")
            .with_boundaries(boundaries)
            .with_unit("PostsRepository")
            .build();

        Self {
            inner,
            request_count,
            request_latency,
        }
    }

    async fn track_method<T, F, E: errors::ProblemResponse + ToString>(
        &self,
        method_name: &str,
        query_summary: &str,
        operation_name: &str,
        target: &str,
        post_id: &str,
        user_id: Option<&str>,
        operation: F,
    ) -> Result<T, E>
    where
        F: std::future::Future<Output = Result<T, E>>,
    {
        let start_time = Instant::now();
        let span = info_span!(
            "",
            "otel.name" = query_summary,
            "otel.kind" = ?SpanKind::Client,
            "db.system.name" = "postgresql",
            "db.operation.name" = operation_name,
            "db.target" = target,
            "method.name" = method_name,
            "post_id" = post_id,
            "user_id" = user_id.unwrap_or_default(),
            "error.message" = field::Empty,
            "error.type" = field::Empty,
        );

        let result = operation.await;
        let status = if result.is_ok() { "success" } else { "error" };
        let mut attributes = vec![
            KeyValue::new("method", method_name.to_string()),
            KeyValue::new("operation", operation_name.to_string()),
        ];

        self.request_latency
            .record(start_time.elapsed().as_secs_f64(), &attributes);

        attributes.push(KeyValue::new("status", status));
        self.request_count.add(1, &attributes);

        if let Err(ref err) = result {
            if err.status_code() == axum::http::StatusCode::INTERNAL_SERVER_ERROR {
                span.record("error.type", "database_error")
                    .set_status(opentelemetry::trace::Status::error(err.to_string()));
                return result;
            }
        }

        span.set_status(opentelemetry::trace::Status::Ok);
        result
    }
}

#[async_trait]
impl<P: PostsRepository + 'static> PostsRepository for ObservablePostsRepository<P> {
    async fn create(&self, post_id: &Ulid, user_id: &Ulid) -> Result<(), DatabaseError> {
        self.track_method(
            "create",
            "INSERT INTO posts",
            "INSERT",
            "posts",
            &post_id.to_string(),
            Some(&user_id.to_string()),
            self.inner.create(post_id, user_id),
        )
        .await
    }

    async fn delete(&self, post_id: &Ulid) -> Result<(), DatabaseError> {
        self.track_method(
            "delete",
            "DELETE FROM posts",
            "DELETE",
            "posts",
            &post_id.to_string(),
            None,
            self.inner.delete(post_id),
        )
        .await
    }
}