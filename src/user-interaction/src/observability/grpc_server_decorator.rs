use prometheus::{HistogramVec, IntCounterVec, Opts, Registry};
use tonic::{Code, Request, Response, Status};
use tracing::{field, info_span, Instrument};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use crate::errors;
use crate::repositories::interaction_repo::InteractionRepository;
use crate::repositories::reply_repo::ReplyRepository;
use crate::services::cache_service::CacheService;
use crate::services::grpc_server::reply_server::{BatchFetchPostInteractionsRequest, BatchFetchPostInteractionsResponse, FetchPostInteractionsRequest, FetchPostInteractionsResponse, FetchReplyByIdRequest, FetchReplyByIdResponse};
use crate::services::grpc_server::reply_server::grpc_reply_service_server::GrpcReplyService;
use crate::services::grpc_server::ReplyServer;

pub struct ObservableReplyServer<S: GrpcReplyService + 'static> {
    inner: S,
    request_count: IntCounterVec,
    request_latency: HistogramVec,
}

impl<S: GrpcReplyService + 'static> ObservableReplyServer<S> {
    pub fn new(inner: S, registry: &Registry) -> Result<Self, errors::AppError> {
        const LATENCY_BUCKETS: &[f64] = &[
            0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
        ];

        let request_count = IntCounterVec::new(
            Opts::new("grpc_requests_total", "Total gRPC requests processed"),
            &["method", "status"],
        )
            .map_err(|e| errors::ObservabilityError::MetricRegistrationError(e.to_string()))?;

        let request_latency = HistogramVec::new(
            prometheus::HistogramOpts::new(
                "grpc_request_duration_seconds",
                "Latency of gRPC methods",
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
                package = "reply_server",
                service = "GrpcReplyService",
                method = method_name
            ),
            "otel.kind" = "server",
            "rpc.system" = "grpc",
            "network.transport" = "udp",
            "rpc.method" = method_name,
            "rcp.service" = format!(
                "{package}.{service}",
                package = "reply_server",
                service = "GrpcReplyService"
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
            match err.code() {
                Code::Internal => {
                    span.record("error.type", "grpc_server_error")
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

#[tonic::async_trait]
impl<
    R: ReplyRepository + 'static,
    I: InteractionRepository + 'static,
    C: CacheService + 'static,
> GrpcReplyService for ObservableReplyServer<ReplyServer<R, I, C>>
{
    async fn fetch_reply_by_id(
        &self,
        request: Request<FetchReplyByIdRequest>,
    ) -> Result<Response<FetchReplyByIdResponse>, Status> {
        self.track_method("fetch_reply_by_id", self.inner.fetch_reply_by_id(request))
            .await
    }

    async fn fetch_post_interactions(
        &self,
        request: Request<FetchPostInteractionsRequest>,
    ) -> Result<Response<FetchPostInteractionsResponse>, Status> {
        self.track_method(
            "fetch_post_interactions",
            self.inner.fetch_post_interactions(request),
        )
            .await
    }

    async fn batch_fetch_post_interactions(
        &self,
        request: Request<BatchFetchPostInteractionsRequest>,
    ) -> Result<Response<BatchFetchPostInteractionsResponse>, Status> {
        self.track_method(
            "batch_fetch_post_interactions",
            self.inner.batch_fetch_post_interactions(request),
        )
            .await
    }
}
