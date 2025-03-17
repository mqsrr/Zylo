use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use crate::repositories::post_repo::PostRepository;
use crate::services::grpc_server::post_server::post_service_server::PostService;
use crate::services::grpc_server::post_server::{BatchPostsRequest, PaginatedPostsResponse, PostRequest, PostResponse, PostsRequest, PostsResponse};
use crate::services::grpc_server::GrpcPostServer;
use opentelemetry::metrics::{Counter, Histogram};
use opentelemetry::{global, KeyValue};
use tonic::{Code, Request, Response, Status};
use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use crate::utils::constants::OTEL_SERVICE_NAME;
use crate::utils::helpers::get_container_id;

pub struct ObservablePostServer<P: PostService + 'static> {
    inner: P,
    request_count: Counter<u64>,
    request_latency: Histogram<f64>,
    active_requests: Arc<AtomicU64>,
    attributes: Vec<KeyValue>,
}

impl<P: PostService + 'static> ObservablePostServer<P> {
    pub fn new(inner: P) -> Self {
        let boundaries = vec![
            0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
        ];

        let meter_provider = global::meter(OTEL_SERVICE_NAME);
        let request_count = meter_provider
            .u64_counter("grpc_server_requests_total")
            .with_description("Total number of gRPC requests")
            .build();

        let request_latency = meter_provider
            .f64_histogram("grpc_server_request_duration_seconds")
            .with_description("Request processing duration")
            .with_boundaries(boundaries)
            .build();
        
        let host_name = get_container_id().unwrap_or(String::from("0.0.0.0"));
        let attributes = vec![
            KeyValue::new("service", OTEL_SERVICE_NAME),
            KeyValue::new("instance", host_name),
            KeyValue::new("env", std::env::var("APP_ENV").unwrap_or(String::from("development")))
        ];
        let active_requests = Arc::new(AtomicU64::new(0));
        let active_requests_clone = active_requests.clone();

        let attributes_clone = attributes.clone();
        meter_provider
            .u64_observable_gauge("grpc_server_active_requests")
            .with_description("Active gRPC requests")
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
            attributes
        }
    }

    async fn track_method<T, F>(
        &self,
        method_name: &str,
        operation: F,
    ) -> Result<Response<T>, Status>
    where
        F: Future<Output = Result<Response<T>, Status>>,
    {
        let start_time = tokio::time::Instant::now();
        self.active_requests.fetch_add(1, Ordering::Relaxed);
        
        let result = operation.await;
        let status = if result.is_ok() { "success" } else { "error" };

        let mut attributes = vec![
            KeyValue::new("method", method_name.to_string()),
        ];
        
        attributes.extend_from_slice(&self.attributes);
        self.request_latency
            .record(start_time.elapsed().as_secs_f64(), &attributes);

        attributes.push(KeyValue::new("status", status));
        self.request_count.add(1, &attributes);
        self.active_requests.fetch_sub(1, Ordering::Relaxed);
        
        let span = Span::current();
        if let Err(ref err) = result {
            let code = err.code();
            span.record("rpc.grpc.status_code", code.to_string());

            match code {
                Code::Unknown
                | Code::DeadlineExceeded
                | Code::Unimplemented
                | Code::Internal
                | Code::Unavailable
                | Code::DataLoss => {
                    span.record("error.type", "grpc_server_error")
                        .set_status(opentelemetry::trace::Status::error(err.to_string()));
                }
                _ => {}
            }

            return result;
        }

        span.set_status(opentelemetry::trace::Status::Ok);
        
        result
    }
}

#[tonic::async_trait]
impl<P> PostService for ObservablePostServer<GrpcPostServer<P>>
where
    P: PostRepository + 'static,
{
    async fn get_post_by_id(&self, request: Request<PostRequest>) -> Result<Response<PostResponse>, Status> {
        self.track_method(
            "GetPostById",
            self.inner.get_post_by_id(request),
        )
            .await
    }

    async fn get_paginated_posts(&self, request: Request<PostsRequest>) -> Result<Response<PaginatedPostsResponse>, Status> {
        self.track_method(
            "GetPaginatedPosts",
            self.inner.get_paginated_posts(request),
        )
            .await
    }

    async fn get_batch_posts(&self, request: Request<BatchPostsRequest>) -> Result<Response<PostsResponse>, Status> {
        self.track_method(
            "GetBatchPosts",
            self.inner.get_batch_posts(request),
        )
            .await
    }
}