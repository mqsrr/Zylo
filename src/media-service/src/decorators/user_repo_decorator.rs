use crate::errors;
use crate::errors::AppError;
use crate::repositories::user_repo::{MongoUserRepository, UserRepository};
use crate::services::cache_service::CacheService;
use async_trait::async_trait;
use mongodb::{ClientSession, Database};
use prometheus::{HistogramVec, IntCounterVec, Opts, Registry};
use std::sync::Arc;
use tokio::time::Instant;
use tracing::{field, info_span};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use ulid::Ulid;
pub struct DecoratedUserRepository<U: UserRepository> {
    user_repo: U,
}

impl DecoratedUserRepository<MongoUserRepository> {
    pub fn new(db: Database) -> Self {
        Self {
            user_repo: MongoUserRepository::new(db),
        }
    }
}

impl<U: UserRepository + 'static> DecoratedUserRepository<U> {
    pub fn cached<C: CacheService>(
        self,
        cache_service: Arc<C>,
    ) -> DecoratedUserRepository<CachedUserRepository<U, C>> {
        let cached_repo = CachedUserRepository::new(self.user_repo, cache_service);

        DecoratedUserRepository {
            user_repo: cached_repo,
        }
    }

    pub fn observable(
        self,
        registry: &Registry,
    ) -> Result<DecoratedUserRepository<ObservableUserRepository<U>>, errors::ObservabilityError> {
        let observable_repo = ObservableUserRepository::new(self.user_repo, registry)?;

        Ok(DecoratedUserRepository {
            user_repo: observable_repo,
        })
    }

    pub fn build(self) -> U {
        self.user_repo
    }
}



#[derive(Clone)]
pub struct ObservableUserRepository<U: UserRepository> {
    inner: U,
    request_count: IntCounterVec,
    request_latency: HistogramVec,
}

impl<U: UserRepository> ObservableUserRepository<U> {
    pub fn new(inner: U, registry: &Registry) -> Result<Self, errors::ObservabilityError> {
        const EXPONENTIAL_SECONDS: &[f64] = &[
            0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
        ];
        let request_count = IntCounterVec::new(
            Opts::new(
                "user_repo_requests_total",
                "Total requests to UserRepository",
            ),
            &["method", "operation", "status"],
        )
        .map_err(|e| errors::ObservabilityError::MetricRegistration(e.to_string()))?;

        let request_latency = HistogramVec::new(
            prometheus::HistogramOpts::new(
                "user_repo_request_duration_seconds",
                "Latency of UserRepository methods",
            )
            .buckets(EXPONENTIAL_SECONDS.to_vec()),
            &["method", "operation"],
        )
        .map_err(|e| errors::ObservabilityError::MetricRegistration(e.to_string()))?;

        registry
            .register(Box::new(request_count.clone()))
            .map_err(|e| errors::ObservabilityError::MetricRegistration(e.to_string()))?;
        registry
            .register(Box::new(request_latency.clone()))
            .map_err(|e| errors::ObservabilityError::MetricRegistration(e.to_string()))?;

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
            "otel.king" = "client",
            "db.system" = "mongodb",
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
impl<U: UserRepository + 'static> UserRepository for ObservableUserRepository<U> {
    async fn start_session(&self) -> Result<ClientSession, AppError> {
        self.track_method(
            "start_session",
            "mongo.start_session users",
            "start_session",
            "sessions",
            self.inner.start_session(),
        )
        .await
    }

    async fn create(&self, user_id: &Ulid) -> Result<(), AppError> {
        self.track_method(
            "create",
            "mongo.insert_one users",
            "insert_one",
            "users",
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
            self.inner.exists(user_id),
        )
        .await
    }

    async fn delete(&self, user_id: &Ulid, session: &mut ClientSession) -> Result<(), AppError> {
        self.track_method(
            "delete",
            "mongo.delete_one users",
            "delete_one",
            "users",
            self.inner.delete(user_id, session),
        )
        .await
    }
}

pub struct CachedUserRepository<U: UserRepository, C: CacheService> {
    inner: U,
    cache_service: Arc<C>,
}

impl<U: UserRepository, C: CacheService> CachedUserRepository<U, C> {
    pub fn new(inner: U, cache_service: Arc<C>) -> Self {
        Self {
            inner,
            cache_service,
        }
    }
}

#[async_trait]
impl<U: UserRepository, C: CacheService> UserRepository for CachedUserRepository<U, C> {
    async fn start_session(&self) -> Result<ClientSession, AppError> {
        self.inner.start_session().await
    }

    async fn create(&self, user_id: &Ulid) -> Result<(), AppError> {
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

        let exists = self.inner.exists(&user_id).await?;
        self.cache_service
            .hset("user-exists", cache_key, &exists)
            .await?;

        Ok(exists)
    }

    async fn delete(&self, user_id: &Ulid, session: &mut ClientSession) -> Result<(), AppError> {
        let cache_key = &user_id.to_string();
        self.inner.delete(user_id, session).await?;

        self.cache_service.hdelete("users", cache_key).await?;
        self.cache_service.hdelete("user-exists", cache_key).await?;

        Ok(())
    }
}
