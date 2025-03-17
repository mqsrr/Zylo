use crate::decorators::trace_server_error;
use crate::errors;
use crate::models::file::{File, FileMetadata, PresignedUrl};
use crate::services::cache_service::CacheService;
use crate::services::s3_service::{S3FileService, S3Service};
use crate::settings::S3Settings;
use crate::utils::helpers::get_container_id;
use async_trait::async_trait;
use opentelemetry::metrics::{Counter, Histogram};
use opentelemetry::{KeyValue, global};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::time::Instant;
use tracing::log::warn;
use tracing::{field, info_span};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use crate::utils::constants::OTEL_SERVICE_NAME;

pub struct DecoratedS3ServiceBuilder<S: S3Service> {
    s3_service: S,
}

impl DecoratedS3ServiceBuilder<S3FileService> {
    pub async fn new(settings: S3Settings) -> Self {
        Self {
            s3_service: S3FileService::new(settings).await,
        }
    }
}

impl<S: S3Service + 'static> DecoratedS3ServiceBuilder<S> {
    pub fn cached<C: CacheService>(
        self,
        cache_service: Arc<C>,
    ) -> DecoratedS3ServiceBuilder<CachedS3Service<S, C>> {
        let cached_repo = CachedS3Service::new(self.s3_service, cache_service);

        DecoratedS3ServiceBuilder {
            s3_service: cached_repo,
        }
    }

    pub fn observable(self) -> DecoratedS3ServiceBuilder<ObservableS3Service<S>> {
        DecoratedS3ServiceBuilder {
            s3_service: ObservableS3Service::new(self.s3_service),
        }
    }

    pub fn build(self) -> S {
        self.s3_service
    }
}

pub struct ObservableS3Service<S: S3Service> {
    inner: S,
    request_count: Counter<u64>,
    request_latency: Histogram<f64>,
    active_requests: Arc<AtomicU64>,
    attributes: Vec<KeyValue>,
}

impl<S: S3Service> ObservableS3Service<S> {
    pub fn new(inner: S) -> Self {
        let boundaries = vec![
            0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
        ];

        let meter_provider = global::meter(OTEL_SERVICE_NAME);
        let request_count = meter_provider
            .u64_counter("s3_requests_total")
            .with_description("Total number of S3 requests")
            .build();

        let request_latency = meter_provider
            .f64_histogram("s3_request_duration_seconds")
            .with_description("Request processing duration")
            .with_boundaries(boundaries)
            .build();

        let host_name = get_container_id().unwrap_or(String::from("0.0.0.0"));
        let attributes = vec![
            KeyValue::new("service", OTEL_SERVICE_NAME),
            KeyValue::new("instance", host_name),
            KeyValue::new(
                "env",
                std::env::var("APP_ENV").unwrap_or(String::from("development")),
            ),
        ];
        let active_requests = Arc::new(AtomicU64::new(0));
        let active_requests_clone = active_requests.clone();

        let attributes_clone = attributes.clone();
        meter_provider
            .u64_observable_gauge("s3_active_requests")
            .with_description("Active S3 requests")
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
        method: &str,
        s3_key: &str,
        operation: F,
    ) -> Result<T, E>
    where
        F: Future<Output = Result<T, E>>,
    {
        let start_time = Instant::now();
        let span = info_span!(
            "",
            "otel.name" = query_summary,
            "otel.kind" = "client",
            "rpc.system" = "aws-api",
            "rpc.service" = "S3",
            "aws.s3.key" = s3_key,
            "rpc.method" = method,
            "method.name" = method_name,
            "error.message" = field::Empty,
            "error.type" = field::Empty,
        );
        
        self.active_requests.fetch_add(1, Ordering::Relaxed);
        let result = operation.await;
        self.active_requests.fetch_sub(1, Ordering::Relaxed);
        
        let status = if result.is_ok() { "success" } else { "error" };
        let mut attributes = vec![KeyValue::new("method", method_name.to_string())];

        self.request_latency
            .record(start_time.elapsed().as_secs_f64(), &attributes);

        attributes.push(KeyValue::new("status", status));
        attributes.extend_from_slice(&self.attributes);

        self.request_count.add(1, &attributes);

        let result = trace_server_error(result, &span, "database_error")?;
        span.set_status(opentelemetry::trace::Status::Ok);

        Ok(result)
    }
}

#[async_trait]
impl<S: S3Service> S3Service for ObservableS3Service<S> {
    async fn upload(&self, file: File) -> Result<FileMetadata, errors::S3Error> {
        self.track_method(
            "upload",
            "s3.upload media file",
            "UploadItem",
            &file.id.to_string(),
            self.inner.upload(file),
        )
        .await
    }

    async fn delete(&self, key: &str) -> Result<(), errors::S3Error> {
        self.track_method(
            "delete",
            "s3.delete media file",
            "DeleteItem",
            key,
            self.inner.delete(key),
        )
        .await
    }

    async fn get_presigned_url(&self, key: &str) -> Result<PresignedUrl, errors::S3Error> {
        self.track_method(
            "get_presigned_url",
            "s3.presign_url media file",
            "GetPresignedURL",
            key,
            self.inner.get_presigned_url(key),
        )
        .await
    }
}

pub struct CachedS3Service<S: S3Service, C: CacheService> {
    inner: S,
    cache_service: Arc<C>,
}

impl<S: S3Service, C: CacheService> CachedS3Service<S, C> {
    pub fn new(inner: S, cache_service: Arc<C>) -> Self {
        Self {
            inner,
            cache_service,
        }
    }
}

#[async_trait]
impl<S: S3Service, C: CacheService> S3Service for CachedS3Service<S, C> {
    async fn upload(&self, file: File) -> Result<FileMetadata, errors::S3Error> {
        self.inner.upload(file).await
    }

    async fn delete(&self, key: &str) -> Result<(), errors::S3Error> {
        self.inner.delete(key).await?;
        let hash_key = "s3-media";
        if self.cache_service.hdelete(hash_key, key).await.is_err() {
            warn!(
                "Could not invalidate hash field {} in {} hash",
                key, hash_key
            );
        }

        Ok(())
    }

    async fn get_presigned_url(&self, key: &str) -> Result<PresignedUrl, errors::S3Error> {
        if let Some(cached_url) = self
            .cache_service
            .hget("s3-media", key)
            .await
            .unwrap_or_default()
        {
            return Ok(cached_url);
        }

        let url = self.inner.get_presigned_url(key).await?;

        let hash_key = "s3-media";
        if self.cache_service.hset(hash_key, key, &url).await.is_err() {
            warn!("Could create hash field {} in {} hash", key, hash_key);
        }

        Ok(url)
    }
}
