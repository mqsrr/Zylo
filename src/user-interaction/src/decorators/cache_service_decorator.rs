use crate::services::cache_service::{CacheService, RedisCacheService};
use crate::utils::helpers::get_container_id;
use crate::{errors, settings};
use async_trait::async_trait;
use opentelemetry::metrics::{Counter, Histogram};
use opentelemetry::{global, KeyValue};
use redis::aio::MultiplexedConnection;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
use tracing::{field, info_span};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use crate::utils::constants::OTEL_SERVICE_NAME;

pub struct DecoratedCacheService<C: CacheService> {
    cache_service: C,
}

impl DecoratedCacheService<RedisCacheService> {
    pub fn new(config: settings::Redis) -> Result<Self, errors::RedisError> {
        Ok(Self {
            cache_service: RedisCacheService::new(config)?,
        })
    }
}

impl<C: CacheService + 'static> DecoratedCacheService<C> {
    pub fn observable(self) -> DecoratedCacheService<ObservableCacheService<C>> {
        DecoratedCacheService {
            cache_service: ObservableCacheService::new(self.cache_service),
        }
    }

    pub fn build(self) -> C {
        self.cache_service
    }
}

pub struct ObservableCacheService<C: CacheService + 'static> {
    inner: C,
    request_count: Counter<u64>,
    request_latency: Histogram<f64>,
    attributes: Vec<KeyValue>
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
            attributes
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
        F: std::future::Future<Output = Result<R, errors::RedisError>>,
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
        if let Err(ref err) = result {
            span.record("error.type", "cache_error")
                .set_status(opentelemetry::trace::Status::error(err.to_string()));

            return result;
        }

        span.set_status(opentelemetry::trace::Status::Ok);
        result
    }
}

#[async_trait]
impl<C: CacheService + 'static> CacheService for ObservableCacheService<C> {
    async fn get_conn(&self) -> Result<MultiplexedConnection, errors::RedisError> {
        self.track_method(
            "get_conn",
            "CONNECT",
            "CONNECT",
            "self",
            self.inner.get_conn(),
        )
        .await
    }

    async fn hfind<T: DeserializeOwned>(
        &self,
        key: &str,
        pattern: &str,
    ) -> Result<Option<T>, errors::RedisError> {
        self.track_method(
            "hfind",
            &format!("HSCAN {} {}", key, pattern),
            "HSCAN",
            key,
            self.inner.hfind(key, pattern),
        )
        .await
    }

    async fn hfind_keys(
        &self,
        key: &str,
        pattern: &str,
    ) -> Result<Vec<String>, errors::RedisError> {
        self.track_method(
            "HSCAN",
            &format!("HSCAN {} {} NOVALUES", key, pattern),
            "HSCAN",
            key,
            self.inner.hfind_keys(key, pattern),
        )
        .await
    }

    async fn hget<T: DeserializeOwned>(
        &self,
        key: &str,
        field: &str,
    ) -> Result<Option<T>, errors::RedisError> {
        self.track_method(
            "hget",
            &format!("HGET {} {}", key, field),
            "HGET",
            key,
            self.inner.hget(key, field),
        )
        .await
    }

    async fn hset<T: Serialize + Sync + Send>(
        &self,
        key: &str,
        field: &str,
        value: &T,
    ) -> Result<(), errors::RedisError> {
        self.track_method(
            "hset",
            &format!("HSET {} {} HEXPIRE", key, field),
            "HSET HEXPIRE",
            key,
            self.inner.hset(key, field, value),
        )
        .await
    }

    async fn hdel(&self, key: &str, fields: &[String]) -> Result<(), errors::RedisError> {
        self.track_method(
            "hdel",
            &format!("HDEL {} {}", key, fields.join(" ")),
            "HDEL",
            key,
            self.inner.hdel(key, fields),
        )
        .await
    }

    async fn pfadd(&self, key: &str, element: &str) -> Result<bool, errors::RedisError> {
        self.track_method(
            "pfadd",
            &format!("PFADD {} {}", key, element),
            "PFADD",
            key,
            self.inner.pfadd(key, element),
        )
        .await
    }

    async fn pfcount(&self, key: &str) -> Result<u64, errors::RedisError> {
        self.track_method(
            "pfcount",
            &format!("PFCOUNT {}", key),
            "PFCOUNT",
            key,
            self.inner.pfcount(key),
        )
        .await
    }

    async fn pfcount_many(
        &self,
        keys: &[String],
    ) -> Result<HashMap<String, u64>, errors::RedisError> {
        let namespace = keys.join(" ");
        self.track_method(
            "pfcount_many",
            &format!("PIPE PFCOUNT {}", &namespace),
            "PIPE PFCOUNT",
            &namespace,
            self.inner.pfcount_many(keys),
        )
        .await
    }

    async fn sadd(&self, key: &str, member: &str) -> Result<bool, errors::RedisError> {
        self.track_method(
            "sadd",
            &format!("SADD {} {}", key, member),
            "SADD",
            key,
            self.inner.sadd(key, member),
        )
        .await
    }

    async fn srem(&self, key: &str, member: &str) -> Result<bool, errors::RedisError> {
        self.track_method(
            "srem",
            &format!("SREM {} {}", key, member),
            "SREM",
            key,
            self.inner.srem(key, member),
        )
        .await
    }

    async fn scard(&self, key: &str) -> Result<u64, errors::RedisError> {
        self.track_method(
            "scard",
            &format!("SCARD {}", key),
            "SCARD",
            key,
            self.inner.scard(key),
        )
        .await
    }

    async fn scard_many(
        &self,
        keys: &[String],
    ) -> Result<HashMap<String, u64>, errors::RedisError> {
        let namespace = keys.join(" ");
        self.track_method(
            "scard_many",
            &format!("PIPE SCARD {}", namespace),
            "PIPE SCARD",
            &namespace,
            self.inner.scard_many(keys),
        )
        .await
    }

    async fn sismember(&self, key: &str, member: &str) -> Result<bool, errors::RedisError> {
        self.track_method(
            "sismember",
            &format!("SISMEMBER {} {}", key, member),
            "SISMEMBER",
            key,
            self.inner.sismember(key, member),
        )
        .await
    }

    async fn sismember_many(
        &self,
        keys: &[String],
        member: &str,
    ) -> Result<HashMap<String, bool>, errors::RedisError> {
        let namespace = keys.join(" ");
        self.track_method(
            "sismember_many",
            &format!("PIPR SISMEMBER {} {}", namespace, member),
            "PIPE SISMEMBER",
            &namespace,
            self.inner.sismember_many(keys, member),
        )
        .await
    }

    async fn del(&self, keys: &[String]) -> Result<(), errors::RedisError> {
        let namespace = keys.join(" ");
        self.track_method(
            "del",
            &format!("DEL {}", namespace),
            "DEL",
            &namespace,
            self.inner.del(keys),
        )
        .await
    }
}
