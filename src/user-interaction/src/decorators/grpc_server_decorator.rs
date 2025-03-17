use crate::services::grpc_server::reply_server::reply_service_server::ReplyService as GrpcReplyService;
use crate::services::grpc_server::reply_server::{
    BatchOfPostInteractionsResponse, GetBatchOfPostInteractionsRequest, GetPostInteractionsRequest,
    GetReplyByIdRequest, PostInteractionsResponse as GrpcPostInteractionResponse,
    ReplyResponse as GrpcReplyResponse,
};
use crate::services::grpc_server::GrpcReplyServer;
use crate::services::post_interactions_service::PostInteractionsService;
use crate::services::reply_service::ReplyService;
use crate::utils::helpers::get_container_id;
use opentelemetry::metrics::{Counter, Histogram};
use opentelemetry::{global, KeyValue};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tonic::{Code, Request, Response, Status};
use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use crate::utils::constants::OTEL_SERVICE_NAME;

pub struct DecoratedGrpcServer<S: GrpcReplyService> {
    reply_server: S,
}

impl<RS: ReplyService + 'static, PS: PostInteractionsService + 'static>
    DecoratedGrpcServer<GrpcReplyServer<RS, PS>>
{
    pub fn new(reply_service: Arc<RS>, post_interactions_service: Arc<PS>) -> Self {
        Self {
            reply_server: GrpcReplyServer::new(reply_service, post_interactions_service),
        }
    }
}

impl<S: GrpcReplyService + 'static> DecoratedGrpcServer<S> {
    pub fn observable(self) -> DecoratedGrpcServer<ObservableGrpcReplyServer<S>> {
        DecoratedGrpcServer {
            reply_server: ObservableGrpcReplyServer::new(self.reply_server),
        }
    }

    pub fn build(self) -> S {
        self.reply_server
    }
}

pub struct ObservableGrpcReplyServer<S: GrpcReplyService + 'static> {
    inner: S,
    request_count: Counter<u64>,
    request_latency: Histogram<f64>,
    active_requests: Arc<AtomicU64>,
    attributes: Vec<KeyValue>,
}

impl<S: GrpcReplyService + 'static> ObservableGrpcReplyServer<S> {
    pub fn new(inner: S) -> Self {
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
        F: std::future::Future<Output = Result<Response<T>, Status>>,
    {
        let start_time = tokio::time::Instant::now();
        self.active_requests.fetch_add(1, Ordering::Relaxed);
        
        let result = operation.await;
        let stop = start_time.elapsed().as_secs_f64();
        let status = if result.is_ok() { "success" } else { "error" };

        let mut attributes = vec![KeyValue::new("method", method_name.to_string())];
        self.request_latency.record(stop, &attributes);

        attributes.push(KeyValue::new("status", status));
        attributes.extend_from_slice(&self.attributes);

        self.request_count.add(1, &attributes);
        let span = Span::current();
        
        self.active_requests.fetch_sub(1, Ordering::Relaxed);
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
impl<S: GrpcReplyService> GrpcReplyService for ObservableGrpcReplyServer<S> {
    async fn get_reply_by_id(
        &self,
        request: Request<GetReplyByIdRequest>,
    ) -> Result<Response<GrpcReplyResponse>, Status> {
        self.track_method("get_reply_by_id", self.inner.get_reply_by_id(request))
            .await
    }

    async fn get_post_interactions(
        &self,
        request: Request<GetPostInteractionsRequest>,
    ) -> Result<Response<GrpcPostInteractionResponse>, Status> {
        self.track_method(
            "get_post_interactions",
            self.inner.get_post_interactions(request),
        )
        .await
    }

    async fn get_batch_of_post_interactions(
        &self,
        request: Request<GetBatchOfPostInteractionsRequest>,
    ) -> Result<Response<BatchOfPostInteractionsResponse>, Status> {
        self.track_method(
            "get_batch_of_post_interactions",
            self.inner.get_batch_of_post_interactions(request),
        )
        .await
    }
}
