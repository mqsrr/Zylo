use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use crate::errors;
use crate::errors::DatabaseError;
use crate::repositories::users_repo::{DeletedPostIds, PostgresUsersRepository, UsersRepository};
use crate::utils::helpers::get_container_id;
use async_trait::async_trait;
use opentelemetry::metrics::{Counter, Histogram};
use opentelemetry::trace::SpanKind;
use opentelemetry::{global, KeyValue};
use sqlx::PgPool;
use tokio::time::Instant;
use tracing::{field, info_span};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use ulid::Ulid;
use crate::utils::constants::OTEL_SERVICE_NAME;

pub struct DecoratedUsersRepository<U: UsersRepository> {
    users_repo: U,
}

impl DecoratedUsersRepository<PostgresUsersRepository> {
    pub fn new(db: PgPool) -> Self {
        Self {
            users_repo: PostgresUsersRepository::new(db),
        }
    }
}

impl<U: UsersRepository + 'static> DecoratedUsersRepository<U> {
    pub fn observable(self) -> DecoratedUsersRepository<ObservableUsersRepository<U>> {
        DecoratedUsersRepository {
            users_repo: ObservableUsersRepository::new(self.users_repo),
        }
    }

    pub fn build(self) -> U {
        self.users_repo
    }
}

#[derive(Clone)]
pub struct ObservableUsersRepository<U: UsersRepository> {
    inner: U,
    request_count: Counter<u64>,
    request_latency: Histogram<f64>,
    active_requests: Arc<AtomicU64>,
    attributes: Vec<KeyValue>,
}

impl<U: UsersRepository> ObservableUsersRepository<U> {
    pub fn new(inner: U) -> Self {
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
        user_id: &str,
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
            "user_id" = user_id,
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

        self.request_latency
            .record(start_time.elapsed().as_secs_f64(), &attributes);
        attributes.push(KeyValue::new("status", status));

        attributes.extend_from_slice(&self.attributes);
        self.request_count.add(1, &attributes);

        if let Err(ref err) = result {
            span.record("error.type", "database_error")
                .set_status(opentelemetry::trace::Status::error(err.to_string()));

            return result;
        }

        span.set_status(opentelemetry::trace::Status::Ok);
        result
    }
}

#[async_trait]
impl<U: UsersRepository + 'static> UsersRepository for ObservableUsersRepository<U> {
    async fn create(&self, user_id: &Ulid) -> Result<(), DatabaseError> {
        self.track_method(
            "create",
            "INSERT INTO users",
            "INSERT",
            "users",
            &user_id.to_string(),
            self.inner.create(user_id),
        )
        .await
    }

    async fn delete(&self, user_id: &Ulid) -> Result<DeletedPostIds, DatabaseError> {
        self.track_method(
            "delete",
            "DELETE FROM users",
            "DELETE",
            "users",
            &user_id.to_string(),
            self.inner.delete(user_id),
        )
        .await
    }
}
