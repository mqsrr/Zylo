use crate::services::grpc_server::reply_server::reply_service_server::ReplyService as GrpcReplyService;
use crate::services::grpc_server::reply_server::{
    BatchOfPostInteractionsResponse, GetBatchOfPostInteractionsRequest, GetPostInteractionsRequest,
    GetReplyByIdRequest, PostInteractionsResponse as GrpcPostInteractionResponse,
    ReplyResponse as GrpcReplyResponse,
};
use crate::services::grpc_server::GrpcReplyServer;
use crate::services::post_interactions_service::PostInteractionsService;
use crate::services::reply_service::{ReplyService};
use opentelemetry::metrics::{Counter, Histogram};
use opentelemetry::trace::SpanKind;
use opentelemetry::{global, KeyValue};
use std::sync::Arc;
use tonic::{Code, Request, Response, Status};
use tracing::{field, info_span, Instrument};
use tracing_opentelemetry::OpenTelemetrySpanExt;

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
}

impl<S: GrpcReplyService + 'static> ObservableGrpcReplyServer<S> {
    pub fn new(inner: S) -> Self {
        let boundaries = vec![
            0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
        ];

        let meter_provider = global::meter("user-interaction");
        let request_count = meter_provider
            .u64_counter("reply_service_requests_total")
            .with_description("Total requests to Reply Service")
            .with_unit("ReplyService")
            .build();

        let request_latency = meter_provider
            .f64_histogram("reply_service_request_duration_seconds")
            .with_description("Latency of Reply Service methods")
            .with_boundaries(boundaries)
            .with_unit("ReplyService")
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
        let service = format!("{}.{}", "reply_server", "ReplyService");
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
impl<S: GrpcReplyService> GrpcReplyService
    for ObservableGrpcReplyServer<S>
{
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
