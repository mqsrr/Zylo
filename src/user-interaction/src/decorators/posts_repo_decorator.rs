use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use crate::errors;
use crate::errors::DatabaseError;
use crate::repositories::posts_repo::{PostgresPostsRepository, PostsRepository};
use crate::utils::helpers::get_container_id;
use async_trait::async_trait;
use opentelemetry::metrics::{Counter, Histogram};
use opentelemetry::{global, KeyValue};
use sqlx::PgPool;
use tokio::time::Instant;
use tracing::{field, info_span};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use ulid::Ulid;
use crate::utils::constants::OTEL_SERVICE_NAME;

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
    active_requests: Arc<AtomicU64>,
    attributes: Vec<KeyValue>,
}

impl<P: PostsRepository> ObservablePostsRepository<P> {
    pub fn new(inner: P) -> Self {
        let boundaries = vec![
            0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
        ];

        let meter_provider = global::meter(OTEL_SERVICE_NAME);
        let request_count = meter_provider
            .u64_counter("db_queries_total")
            .with_description("Total number of database queries")
            .build();

        let request_latency = meter_provider
            .f64_histogram("db_query_duration_seconds")
            .with_description("Query execution duration")
            .with_boundaries(boundaries)
            .build();

        let host_name = get_container_id().unwrap_or(String::from("0.0.0.0"));
        let attributes = vec![
            KeyValue::new("service", OTEL_SERVICE_NAME),
            KeyValue::new("instance", host_name),
            KeyValue::new("db", "postgres"),
            KeyValue::new("env", std::env::var("APP_ENV").unwrap_or(String::from("development"))),
        ];
        let active_requests = Arc::new(AtomicU64::new(0));
        let active_requests_clone = active_requests.clone();

        let attributes_clone = attributes.clone();
        meter_provider
            .u64_observable_gauge("db_connections")
            .with_description("Active database connections")
            .with_callback(move |observer| {
                let value = active_requests_clone.load(Ordering::Relaxed);
                observer.observe(value, &attributes_clone);
            })
            .build();

        Self {
            inner,
            request_count,
            request_latency,
            active_requests,
            attributes,
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
            "otel.kind" = "client",
            "db.system.name" = "postgresql",
            "db.operation.name" = operation_name,
            "db.target" = target,
            "method.name" = method_name,
            "post_id" = post_id,
            "user_id" = user_id.unwrap_or_default(),
            "error.message" = field::Empty,
            "error.type" = field::Empty,
        );
        
        self.active_requests.fetch_add(1, Ordering::Relaxed);
        let result = operation.await;
        self.active_requests.fetch_sub(1, Ordering::Relaxed);
        
        let status = if result.is_ok() { "success" } else { "error" };
        let mut attributes = vec![
            KeyValue::new("method", method_name.to_string()),
            KeyValue::new("query_type", operation_name.to_string()),
            KeyValue::new("table", target.to_string()),
        ];

        self.request_latency.record(start_time.elapsed().as_secs_f64(), &attributes);
        attributes.push(KeyValue::new("status", status));
        
        attributes.extend_from_slice(&self.attributes);
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
