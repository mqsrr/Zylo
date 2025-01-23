use prometheus::{HistogramVec, IntCounterVec, Opts, Registry};
use tonic::{Code, Request, Response, Status};
use tracing::{field, info_span, Instrument};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use crate::errors;
use crate::repositories::post_repo::PostRepository;
use crate::services::grpc_server::GrpcPostServer;
use crate::services::grpc_server::post_server::post_service_server::PostService;
use crate::services::grpc_server::post_server::{BatchPostsRequest, PaginatedPostsResponse, PostRequest, PostResponse, PostsRequest, PostsResponse};

pub struct ObservablePostServer<P: PostService + 'static> {
    inner: P,
    request_count: IntCounterVec,
    request_latency: HistogramVec,
}

impl<P: PostService + 'static> ObservablePostServer<P> {
    pub fn new(inner: P, registry: &Registry) -> Result<Self, errors::ObservabilityError> {
        const LATENCY_BUCKETS: &[f64] = &[
            0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
        ];

        let request_count = IntCounterVec::new(
            Opts::new("grpc_requests_total", "Total gRPC requests processed"),
            &["method", "status"],
        )
            .map_err(|e| errors::ObservabilityError::MetricRegistration(e.to_string()))?;

        let request_latency = HistogramVec::new(
            prometheus::HistogramOpts::new(
                "grpc_request_duration_seconds",
                "Latency of gRPC methods",
            )
                .buckets(LATENCY_BUCKETS.to_vec()),
            &["method"],
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

    async fn track_method<T, F>(
        &self,
        method_name: &str,
        operation: F,
    ) -> Result<Response<T>, Status>
    where
        F: std::future::Future<Output = Result<Response<T>, Status>>,
    {
        let start_time = tokio::time::Instant::now();
        let span = info_span!(
            "",
            "otel.name" = format!(
                "{package}.{service}/{method}",
                package = "post_server",
                service = "PostService",
                method = method_name
            ),
            "otel.kind" = "server",
            "rpc.system" = "grpc",
            "network.transport" = "udp",
            "rpc.method" = method_name,
            "rcp.service" = format!(
                "{package}.{service}",
                package = "post_server",
                service = "PostService"
            ),
            "error.type" = field::Empty
        );

        let result = operation.instrument(span.clone()).await;

        let status_label = if result.is_ok() { "success" } else { "error" };
        self.request_count
            .with_label_values(&[method_name, status_label])
            .inc();

        self.request_latency
            .with_label_values(&[method_name])
            .observe(start_time.elapsed().as_secs_f64());

        if let Err(ref err) = result {
            if err.code() == Code::Internal {
                span.record("error.type", "grpc_server_error")
                    .set_status(opentelemetry::trace::Status::error(err.to_string()));
                return result;
            }
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
            "get_post_by_id",
            self.inner.get_post_by_id(request),
        )
            .await
    }

    async fn get_users_posts(&self, request: Request<PostsRequest>) -> Result<Response<PaginatedPostsResponse>, Status> {
        self.track_method(
            "get_users_posts",
            self.inner.get_users_posts(request),
        )
            .await
    }

    async fn get_batch_posts(&self, request: Request<BatchPostsRequest>) -> Result<Response<PostsResponse>, Status> {
        self.track_method(
            "get_batch_posts",
            self.inner.get_batch_posts(request),
        )
            .await
    }
}