use crate::decorators::trace_server_error;
use crate::errors;
use crate::services::cache_service::CacheService;
use crate::utils::constants::OTEL_SERVICE_NAME;
use crate::utils::helpers::get_container_id;
use async_trait::async_trait;
use opentelemetry::metrics::{Counter, Histogram};
use opentelemetry::{KeyValue, global};
use redis::aio::MultiplexedConnection;
use serde::Serialize;
use serde::de::DeserializeOwned;
use tracing::{field, info_span};
use tracing_opentelemetry::OpenTelemetrySpanExt;

pub struct ObservableCacheService<C: CacheService + 'static> {
    inner: C,
    request_count: Counter<u64>,
    request_latency: Histogram<f64>,
    attributes: Vec<KeyValue>,
}

impl<T: CacheService + 'static> ObservableCacheService<T> {
    pub fn new(inner: T) -> Self {
        let boundaries = vec![
            0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
        ];

        let meter_provider = global::meter(OTEL_SERVICE_NAME);
        let request_count = meter_provider
            .u64_counter("cache_operations_total")
            .with_description("Total cache operations (set/get/delete)")
            .build();

        let request_latency = meter_provider
            .f64_histogram("cache_operation_duration_seconds")
            .with_description("Time taken for cache operations")
            .with_boundaries(boundaries)
            .build();

        let host_name = get_container_id().unwrap_or(String::from("0.0.0.0"));
        let attributes = vec![
            KeyValue::new("service", OTEL_SERVICE_NAME),
            KeyValue::new("instance", host_name),
            KeyValue::new("cache", "redis"),
            KeyValue::new("env", std::env::var("APP_ENV").unwrap_or(String::from("development")))
        ];

        Self {
            inner,
            request_count,
            request_latency,
            attributes,
        }
    }

    async fn track_method<F, R>(
        &self,
        method_name: &str,
        query_summary: &str,
        operation_name: &str,
        namespace: &str,
        operation: F,
    ) -> Result<R, errors::RedisError>
    where
        F: Future<Output = Result<R, errors::RedisError>>,
    {
        let start_time = tokio::time::Instant::now();
        let span = info_span!(
            "",
            "otel.name" = query_summary,
            "otel.kind" = "client",
            "db.system.name" = "redis",
            "db.operation.name" = operation_name,
            "db.namespace" = namespace,
            "method.name" = method_name,
            "network.transport" = "unix",
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
        attributes.extend_from_slice(&self.attributes);

        self.request_count.add(1, &attributes);

        let result = trace_server_error(result, &span, "cache_error")?;
        span.set_status(opentelemetry::trace::Status::Ok);

        Ok(result)
    }
}

#[async_trait]
impl<C: CacheService + 'static> CacheService for ObservableCacheService<C> {
    async fn open_redis_connection(&self) -> Result<MultiplexedConnection, errors::RedisError> {
        self.track_method(
            "open_redis_connection",
            "CONNECT redis",
            "CONNECT",
            "redis",
            self.inner.open_redis_connection(),
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
            &format!("redis.HGET {key} {field}"),
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
            &format!("redis.HSET {key} {field}"),
            "HSET",
            key,
            self.inner.hset(key, field, value),
        )
        .await
    }

    async fn hdelete(&self, key: &str, field: &str) -> Result<(), errors::RedisError> {
        self.track_method(
            "delete",
            &format!("redis.HDEL {key} {field}"),
            "HDEL",
            key,
            self.inner.hdelete(key, field),
        )
        .await
    }

    async fn hdelete_all(&self, key: &str, pattern: &str) -> Result<(), errors::RedisError> {
        self.track_method(
            "delete_all",
            &format!("redis.HSCAN {key} 0 {pattern} HDEL {key} {{retrieved fields}}"),
            "HSCAN; HDEL",
            key,
            self.inner.hdelete_all(key, pattern),
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
            &format!("redis.HGET {key} {field}"),
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
            &format!("redis.HDEL {key} {field}"),
            "HDEL",
            key,
            self.inner.hdel_with_conn(conn, key, field),
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
            &format!("redis.HSCAN {key} 0 {pattern} HDEL {key} {{retrieved fields}}"),
            "HSCAN; HDEL",
            key,
            self.inner.delete_all_with_conn(conn, key, pattern),
        )
        .await
    }
}