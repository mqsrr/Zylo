use crate::errors;
use crate::errors::ProblemResponse;
use crate::services::cache_service::CacheService;
use async_trait::async_trait;
use prometheus::{HistogramVec, IntCounterVec, Opts, Registry};
use redis::aio::MultiplexedConnection;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tracing::{field, info_span};
use tracing_opentelemetry::OpenTelemetrySpanExt;

pub struct ObservableCacheService<C: CacheService + 'static> {
    inner: C,
    request_count: IntCounterVec,
    request_latency: HistogramVec,
}

impl<T: CacheService + 'static> ObservableCacheService<T> {
    pub fn new(inner: T, registry: &Registry) -> Result<Self, errors::AppError> {
        const LATENCY_BUCKETS: &[f64] = &[
            0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
        ];

        let request_count = IntCounterVec::new(
            Opts::new("cache_requests_total", "Total cache requests processed"),
            &["method", "status"],
        )
        .map_err(|e| errors::ObservabilityError::MetricRegistrationError(e.to_string()))?;

        let request_latency = HistogramVec::new(
            prometheus::HistogramOpts::new(
                "cache_request_duration_seconds",
                "Latency of cache methods",
            )
            .buckets(LATENCY_BUCKETS.to_vec()),
            &["method"],
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

    async fn track_method<F, R>(
        &self,
        method_name: &str,
        query_summary: &str,
        operation_name: &str,
        target: &str,
        operation: F,
    ) -> Result<R, errors::RedisError>
    where
        F: std::future::Future<Output = Result<R, errors::RedisError>>,
    {
        let start_time = tokio::time::Instant::now();
        let span = info_span!(
            "",
            "otel.name" = query_summary,
            "db.system" = "cache",
            "db.operation.name" = operation_name,
            "db.collection.name" = target,
            "error.type" = field::Empty
        );

        let result = operation.await;

        let status_label = if result.is_ok() { "success" } else { "error" };
        self.request_count
            .with_label_values(&[method_name, status_label])
            .inc();

        self.request_latency
            .with_label_values(&[method_name])
            .observe(start_time.elapsed().as_secs_f64());

        if let Err(ref err) = result {
            match err.status_code() {
                axum::http::StatusCode::INTERNAL_SERVER_ERROR => {
                    span.record("error.type", "cache_error")
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
impl<C: CacheService + 'static> CacheService for ObservableCacheService<C> {
    async fn open_redis_connection(&self) -> Result<MultiplexedConnection, errors::RedisError> {
        self.track_method(
            "open_redis_connection",
            "CONNECT to redis",
            "CONNECT",
            "redis",
            self.inner.open_redis_connection(),
        )
        .await
    }

    async fn hfind<U: DeserializeOwned>(
        &self,
        key: &str,
        pattern: &str,
    ) -> Result<Option<U>, errors::RedisError> {
        self.track_method(
            "hfind",
            &format!("CONNECT HSCAN in {} hash based on {} pattern", key, pattern),
            "HSCAN",
            key,
            self.inner.hfind(key, pattern),
        )
        .await
    }

    async fn hget<U: DeserializeOwned>(
        &self,
        key: &str,
        field: &str,
    ) -> Result<Option<U>, errors::RedisError> {
        self.track_method(
            "hget",
            &format!("HGET {} from {}", field, key),
            "HGET",
            key,
            self.inner.hget(key, field),
        )
        .await
    }

    async fn hset<U: Serialize + Sync + Send>(
        &self,
        key: &str,
        field: &str,
        value: &U,
    ) -> Result<(), errors::RedisError> {
        self.track_method(
            "hset",
            &format!("HSET {} into {}", field, key),
            "HSET",
            key,
            self.inner.hset(key, field, value),
        )
        .await
    }

    async fn sadd(&self, key: &str, member: &str) -> Result<(), errors::RedisError> {
        self.track_method(
            "sadd",
            &format!("SADD {} into {}", member, key),
            "SADD",
            key,
            self.inner.sadd(key, member),
        )
        .await
    }

    async fn sismember(&self, key: &str, member: &str) -> Result<bool, errors::RedisError> {
        self.track_method(
            "sismember",
            &format!("SISMEMBER {} of {}", member, key),
            "SISMEMBER",
            key,
            self.inner.sismember(key, member),
        )
        .await
    }

    async fn srem(&self, key: &str, member: &str) -> Result<(), errors::RedisError> {
        self.track_method(
            "srem",
            &format!("SREM {} from {}", member, key),
            "SREM",
            key,
            self.inner.srem(key, member),
        )
        .await
    }

    async fn delete(&self, key: &str, field: &str) -> Result<(), errors::RedisError> {
        self.track_method(
            "delete",
            &format!("HDEL {} from {}", field, key),
            "HDEL",
            key,
            self.inner.delete(key, field),
        )
        .await
    }

    async fn delete_all(&self, key: &str, pattern: &str) -> Result<(), errors::RedisError> {
        self.track_method(
            "delete_all",
            &format!("HDEL all from {} based on {} pattern", key, pattern),
            "HDEL",
            key,
            self.inner.delete_all(key, pattern),
        )
        .await
    }

    async fn sismember_with_conn(
        &self,
        conn: &mut MultiplexedConnection,
        key: &str,
        member: &str,
    ) -> Result<bool, errors::RedisError> {
        self.track_method(
            "sismember",
            &format!("SISMEMBER {} of {}", member, key),
            "SISMEMBER",
            key,
            self.inner.sismember(key, member),
        )
        .await
    }

    async fn sadd_with_conn(
        &self,
        conn: &mut MultiplexedConnection,
        key: &str,
        member: &str,
    ) -> Result<(), errors::RedisError> {
        self.track_method(
            "sadd_with_conn",
            &format!("SISMEMBER {} of {}", member, key),
            "SISMEMBER",
            key,
            self.inner.sadd_with_conn(conn, key, member),
        )
        .await
    }

    async fn srem_with_conn(
        &self,
        conn: &mut MultiplexedConnection,
        key: &str,
        member: &str,
    ) -> Result<(), errors::RedisError> {
        self.track_method(
            "srem_with_conn",
            &format!("SREM {} of {}", member, key),
            "SREM",
            key,
            self.inner.srem_with_conn(conn, key, member),
        )
        .await
    }

    async fn hincr_with_conn(
        &self,
        conn: &mut MultiplexedConnection,
        key: &str,
        field: &str,
        increment: i32,
    ) -> Result<(), errors::RedisError> {
        self.track_method(
            "hincr_with_conn",
            &format!("HINCR {} of {} on {}", key, field, increment),
            "HINCR",
            key,
            self.inner.hincr_with_conn(conn, key, field, increment),
        )
        .await
    }

    async fn hget_with_conn<T: DeserializeOwned>(
        &self,
        conn: &mut MultiplexedConnection,
        key: &str,
        field: &str,
    ) -> Result<Option<T>, errors::RedisError> {
        self.track_method(
            "hget_with_conn",
            &format!("HGET {} from {}", field, key),
            "HGET",
            key,
            self.inner.hget_with_conn::<T>(conn, key, field),
        )
        .await
    }

    async fn hdel_with_conn(
        &self,
        conn: &mut MultiplexedConnection,
        key: &str,
        field: &str,
    ) -> Result<(), errors::RedisError> {
        self.track_method(
            "hdel_with_conn",
            &format!("HDEL {} from {}", field, key),
            "HDEL",
            key,
            self.inner.hdel_with_conn(conn, key, field),
        )
        .await
    }

    async fn smembers_with_conn(
        &self,
        conn: &mut MultiplexedConnection,
        key: &str,
    ) -> Result<Vec<String>, errors::RedisError> {
        self.track_method(
            "smembers_with_conn",
            &format!("SMEMBERS of {}", key),
            "SISMEMBER",
            key,
            self.inner.smembers_with_conn(conn, key),
        )
        .await
    }

    async fn delete_all_with_conn(
        &self,
        conn: &mut MultiplexedConnection,
        key: &str,
        pattern: &str,
    ) -> Result<(), errors::RedisError> {
        self.track_method(
            "delete_all_with_conn",
            &format!("HDEL all from {} based on {} pattern", key, pattern),
            "HDEL",
            key,
            self.inner.delete_all_with_conn(conn, key, pattern),
        )
        .await
    }
}
