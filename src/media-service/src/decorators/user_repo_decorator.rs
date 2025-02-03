use crate::decorators::trace_server_error;
use crate::errors;
use crate::errors::AppError;
use crate::repositories::user_repo::{MongoUserRepository, UsersRepository};
use crate::services::cache_service::CacheService;
use async_trait::async_trait;
use mongodb::Database;
use opentelemetry::metrics::{Counter, Histogram};
use opentelemetry::trace::SpanKind;
use opentelemetry::{global, KeyValue};
use std::sync::Arc;
use tokio::time::Instant;
use tracing::{field, info_span};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use ulid::Ulid;

pub struct DecoratedUserRepository<U: UsersRepository> {
    user_repo: U,
}

impl DecoratedUserRepository<MongoUserRepository> {
    pub fn new(db: Database) -> Self {
        Self {
            user_repo: MongoUserRepository::new(db),
        }
    }
}

impl<U: UsersRepository + 'static> DecoratedUserRepository<U> {
    pub fn cached<C: CacheService>(
        self,
        cache_service: Arc<C>,
    ) -> DecoratedUserRepository<CachedUserRepository<U, C>> {
        DecoratedUserRepository {
            user_repo: CachedUserRepository::new(self.user_repo, cache_service),
        }
    }

    pub fn observable(self) -> DecoratedUserRepository<ObservableUsersRepository<U>> {
        DecoratedUserRepository {
            user_repo: ObservableUsersRepository::new(self.user_repo),
        }
    }

    pub fn build(self) -> U {
        self.user_repo
    }
}

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

        let meter_provider = global::meter("media-service");
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

        let result = trace_server_error(result, &span, "database_error")?;
        span.set_status(opentelemetry::trace::Status::Ok);

        Ok(result)
    }
}

#[async_trait]
impl<U: UsersRepository + 'static> UsersRepository for ObservableUsersRepository<U> {
    async fn create(&self, user_id: Ulid) -> Result<(), AppError> {
        self.track_method(
            "create",
            "mongo.insert_one users",
            "insert_one",
            "users",
            Some(&user_id.to_string()),
            self.inner.create(user_id),
        )
        .await
    }

    async fn exists(&self, user_id: &Ulid) -> Result<bool, AppError> {
        self.track_method(
            "exists",
            "mongo.count_documents users",
            "count_documents",
            "users",
            Some(&user_id.to_string()),
            self.inner.exists(user_id),
        )
        .await
    }

    async fn delete(&self, user_id: &Ulid) -> Result<(), AppError> {
        self.track_method(
            "delete",
            "mongo.delete_one users",
            "delete_one",
            "users",
            Some(&user_id.to_string()),
            self.inner.delete(user_id),
        )
        .await
    }
}

pub struct CachedUserRepository<U: UsersRepository, C: CacheService> {
    inner: U,
    cache_service: Arc<C>,
}

impl<U: UsersRepository, C: CacheService> CachedUserRepository<U, C> {
    pub fn new(inner: U, cache_service: Arc<C>) -> Self {
        Self {
            inner,
            cache_service,
        }
    }
}

#[async_trait]
impl<U: UsersRepository, C: CacheService> UsersRepository for CachedUserRepository<U, C> {
    async fn create(&self, user_id: Ulid) -> Result<(), AppError> {
        self.inner.create(user_id).await
    }

    async fn exists(&self, user_id: &Ulid) -> Result<bool, AppError> {
        let cache_key = &user_id.to_string();
        if let Some(cached_bool) = self
            .cache_service
            .hget::<bool>("user-exists", cache_key)
            .await?
        {
            return Ok(cached_bool);
        }

        let exists = self.inner.exists(user_id).await?;
        self.cache_service
            .hset("user-exists", cache_key, &exists)
            .await?;

        Ok(exists)
    }

    async fn delete(&self, user_id: &Ulid) -> Result<(), AppError> {
        let cache_key = &user_id.to_string();
        self.inner.delete(user_id).await?;

        self.cache_service.hdelete("users", cache_key).await?;
        self.cache_service.hdelete("user-exists", cache_key).await?;

        Ok(())
    }
}
