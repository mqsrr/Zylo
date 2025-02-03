use opentelemetry::{global, KeyValue};
use opentelemetry::metrics::{Counter, Histogram};
use opentelemetry::trace::SpanKind;
use tonic::{Code, Request, Response, Status};
use tracing::{field, info_span, Instrument};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use crate::repositories::post_repo::PostRepository;
use crate::services::grpc_server::GrpcPostServer;
use crate::services::grpc_server::post_server::post_service_server::PostService;
use crate::services::grpc_server::post_server::{BatchPostsRequest, PaginatedPostsResponse, PostRequest, PostResponse, PostsRequest, PostsResponse};

pub struct ObservablePostServer<P: PostService + 'static> {
    inner: P,
    request_count: Counter<u64>,
    request_latency: Histogram<f64>,
}

impl<P: PostService + 'static> ObservablePostServer<P> {
    pub fn new(inner: P) -> Self {
        let boundaries = vec![
            0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
        ];

        let meter_provider = global::meter("media-service");
        let request_count = meter_provider
            .u64_counter("post_server_requests_total")
            .with_description("Total requests to Grpc Posts Server")
            .with_unit("PostService")
            .build();

        let request_latency = meter_provider
            .f64_histogram("post_server_request_duration_seconds")
            .with_description("Latency of Grpc Posts Server methods")
            .with_boundaries(boundaries)
            .with_unit("PostService")
            .build();

        Self {
            inner,
            request_count,
            request_latency,
        }
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
        let service = format!("{}.{}", "post_service", "PostService");
        let span = info_span!(
            "",
            "otel.name" = format!("{}/{}",service,method_name),
            "otel.kind" = ?SpanKind::Server,
            "rpc.system" = "grpc",
            "server.address" = "127.0.0.1",
            "server.port" = "50051",
            "network.transport" = "udp",
            "rpc.method" = method_name,
            "rcp.service" = &service,
            "rpc.grpc.status_code" = field::Empty,
            "error.type" = field::Empty
        );

        let result = operation.instrument(span.clone()).await;
        let status = if result.is_ok() { "success" } else { "error" };

        let mut attributes = vec![
            KeyValue::new("method", method_name.to_string()),
            KeyValue::new("operation", method_name.to_string()),
        ];

        self.request_latency
            .record(start_time.elapsed().as_secs_f64(), &attributes);

        attributes.push(KeyValue::new("status", status));
        self.request_count.add(1, &attributes);

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