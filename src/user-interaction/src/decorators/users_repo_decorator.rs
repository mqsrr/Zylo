use crate::errors;
use crate::errors::DatabaseError;
use crate::repositories::users_repo::{DeletedPostIds, PostgresUsersRepository, UsersRepository};
use async_trait::async_trait;
use opentelemetry::metrics::{Counter, Histogram};
use opentelemetry::trace::{SpanKind};
use opentelemetry::{global, KeyValue};
use sqlx::PgPool;
use tokio::time::Instant;
use tracing::{field, info_span};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use ulid::Ulid;

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
}

impl<U: UsersRepository> ObservableUsersRepository<U> {
    pub fn new(inner: U) -> Self {
        let boundaries = vec![
            0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
        ];

        let meter_provider = global::meter("user-interaction");
        let request_count = meter_provider
            .u64_counter("users_repo_requests_total")
            .with_description("Total requests to UsersRepository")
            .with_unit("UsersRepository")
            .build();

        let request_latency = meter_provider
            .f64_histogram("users_repo_request_duration_seconds")
            .with_description("Latency of UsersRepository methods")
            .with_boundaries(boundaries)
            .with_unit("UsersRepository")
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
