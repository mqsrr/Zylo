use crate::auth::authorization_middleware;
use crate::models::app_state::AppState;
use crate::repositories::interaction_repo::InteractionRepository;
use crate::{routes};
use crate::services::amq_client::AmqClient;
use crate::services::grpc_server::reply_server::reply_service_server::{
    ReplyService as GrpcReplyServer, ReplyServiceServer as GrpcReplyServiceServer,
};
use crate::services::post_interactions_service::PostInteractionsService;
use crate::services::reply_service::ReplyService;
use crate::utils::constants::{OTEL_SERVICE_NAME, REQUEST_ID_HEADER};
use crate::utils::helpers::get_container_id;
use axum::extract::{MatchedPath, Request as AxRequest, State};
use axum::http::{header, HeaderName, Request};
use axum::middleware;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::Router;
use opentelemetry::metrics::{Counter, Histogram};
use opentelemetry::trace::TracerProvider as _;
use opentelemetry::{global, KeyValue};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_http::HeaderExtractor;
use opentelemetry_otlp::{LogExporter, MetricExporter, SpanExporter, WithExportConfig};
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_sdk::Resource;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::signal;
use tokio::time::Instant;
use tonic::transport::Server;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::sensitive_headers::SetSensitiveHeadersLayer;
use tower_http::trace;
use tower_http::trace::DefaultOnRequest;
use tracing::log::{error, info};
use tracing::{field, info_span};
use tracing_opentelemetry::{OpenTelemetryLayer, OpenTelemetrySpanExt};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter};



#[derive(Clone)]
pub struct ServerMetrics {
    pub request_counter: Counter<u64>,
    pub request_latency: Histogram<f64>,
    pub active_requests: Arc<AtomicU64>,
    pub attributes: Vec<KeyValue>,
}

impl ServerMetrics {
    pub fn new(service_name: &'static str) -> Self {
        let meter = global::meter(service_name);
        let boundaries = vec![
            0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
        ];

        let request_counter = meter
            .u64_counter("http_server_requests_total")
            .with_description("Total number of HTTP requests")
            .build();

        let request_latency = meter
            .f64_histogram("http_server_request_duration_seconds")
            .with_description("HTTP request duration")
            .with_boundaries(boundaries)
            .build();

        let active_requests = Arc::new(AtomicU64::new(0));
        let active_requests_clone = active_requests.clone();

        let host_name = get_container_id().unwrap_or(String::from("0.0.0.0"));
        let attributes = vec![
            KeyValue::new("service", OTEL_SERVICE_NAME),
            KeyValue::new("instance", host_name),
        ];
        let attributes_clone = attributes.clone();

        meter
            .u64_observable_gauge("http_server_active_requests")
            .with_description("Active HTTP requests")
            .with_callback(move |observer| {
                let value = active_requests_clone.load(Ordering::Relaxed);
                observer.observe(value, &attributes_clone);
            })
            .build();

        Self {
            request_counter,
            request_latency,
            active_requests,
            attributes,
        }
    }
}

fn get_resource() -> Resource {
    static RESOURCE: OnceLock<Resource> = OnceLock::new();
    RESOURCE
        .get_or_init(|| {
            Resource::builder()
                .with_service_name(OTEL_SERVICE_NAME)
                .build()
        })
        .clone()
}

pub fn init_traces(endpoint: &str) -> SdkTracerProvider {
    let exporter = SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .build()
        .expect("Failed to create span exporter");

    let provider = SdkTracerProvider::builder()
        .with_resource(get_resource())
        .with_batch_exporter(exporter)
        .build();

    global::set_tracer_provider(provider.clone());
    global::set_text_map_propagator(TraceContextPropagator::new());
    provider
}

pub fn init_tracing(logger_provider: &SdkLoggerProvider, tracer_provider: &SdkTracerProvider) {
    tracing_subscriber::registry()
        .with(OpenTelemetryTracingBridge::new(logger_provider))
        .with(OpenTelemetryLayer::new(
            tracer_provider.tracer("tracing-jaeger"),
        ))
        .with(fmt::layer().pretty())
        .with(EnvFilter::from_default_env())
        .init();
}

pub fn init_metrics(endpoint: &str) -> SdkMeterProvider {
    let exporter = MetricExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .build()
        .expect("Failed to create metric exporter");

    let reader = opentelemetry_sdk::metrics::PeriodicReader::builder(exporter)
        .with_interval(Duration::from_secs(10))
        .build();

    let provider = SdkMeterProvider::builder()
        .with_reader(reader.clone())
        .with_resource(get_resource())
        .build();

    global::set_meter_provider(provider.clone());
    provider
}

pub fn init_logs(endpoint: &str) -> SdkLoggerProvider {
    let exporter = LogExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .build()
        .expect("Failed to create log exporter");

    SdkLoggerProvider::builder()
        .with_resource(get_resource())
        .with_batch_exporter(exporter)
        .build()
}

async fn track_metrics(
    State(metrics): State<Arc<ServerMetrics>>,
    req: AxRequest,
    next: Next,
) -> impl IntoResponse {
    let path = if let Some(matched_path) = req.extensions().get::<MatchedPath>() {
        matched_path.as_str().to_owned()
    } else {
        req.uri().path().to_owned()
    };

    let method = req.method().clone();
    let start = Instant::now();
    metrics.active_requests.fetch_add(1, Ordering::Relaxed);

    let response = next.run(req).await;

    metrics.active_requests.fetch_sub(1, Ordering::Relaxed);
    let latency = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();

    let mut labels = vec![
        KeyValue::new("method", method.to_string()),
        KeyValue::new("path", path),
        KeyValue::new("status", status),
    ];
    labels.extend_from_slice(&metrics.attributes);

    metrics.request_counter.add(1, &labels);
    metrics.request_latency.record(latency, &labels);
    response
}

pub async fn create_router<A, I, RS, PS>(app_state: AppState<A, I, RS, PS>) -> Router
where
    A: AmqClient + 'static,
    I: InteractionRepository + 'static,
    RS: ReplyService + 'static,
    PS: PostInteractionsService + 'static,
{
    let x_request_id = HeaderName::from_static(REQUEST_ID_HEADER);
    let middleware = ServiceBuilder::new()
        .layer(SetRequestIdLayer::new(
            x_request_id.clone(),
            MakeRequestUuid,
        ))
        .layer(
            trace::TraceLayer::new_for_http()
                .make_span_with({
                    let server_port = app_state.config.global.server_port.to_string();
                    move |request: &Request<_>| {
                        let request_id = request.headers().get(REQUEST_ID_HEADER);
                        let request_uri = request.uri();
                        let request_path = request.extensions().get::<MatchedPath>().unwrap().as_str();
                        let method = request.method().to_string();

                        let parent_cx = global::get_text_map_propagator(|propagator| {
                            propagator.extract(&HeaderExtractor(request.headers()))
                        });

                        let span = info_span!(
                            "",
                            "otel.name" = format!("{} {}", &method, &request_path),
                            "http.request.method" = &method,
                            "http.request.method_original" = &method,
                            "server.address" = "0.0.0.0",
                            "server.port" = server_port,
                            "otel.kind" = "server",
                            "http.request.id" = field::Empty,
                            "url.query" = field::Empty,
                            "http.response.status_code" = field::Empty,
                            "http.response.content_length" = field::Empty,
                        );
                        span.set_parent(parent_cx);

                        if let Some(query) = request_uri.query() {
                            span.record("url.query", query);
                        }

                        if let Some(request_id) = request_id {
                            span.record("http.request.id", request_id.to_str().unwrap_or_default());
                        }

                        span
                    }
                })
                .on_request(DefaultOnRequest::new().level(tracing::Level::INFO))
                .on_response(
                    trace::DefaultOnResponse::new()
                        .level(tracing::Level::INFO)
                        .include_headers(true),
                )
                .on_failure(trace::DefaultOnFailure::new().level(tracing::Level::ERROR)),
        )
        .layer(PropagateRequestIdLayer::new(x_request_id));

    Router::new()
        .merge(routes::reply::create_router(app_state.clone()))
        .merge(routes::interaction::create_router(app_state.clone()))
        .layer(middleware)
        .layer(middleware::from_fn_with_state(
            app_state.config.auth.clone(),
            authorization_middleware,
        ))
        .layer(middleware::from_fn_with_state(
            Arc::new(ServerMetrics::new(OTEL_SERVICE_NAME)),
            track_metrics,
        ))
        .layer(SetSensitiveHeadersLayer::new(std::iter::once(
            header::AUTHORIZATION,
        )))
        .layer(CorsLayer::permissive())
}

pub async fn run_app<A, I, RS, PS>(
    app_state: AppState<A, I, RS, PS>,
    grpc_server: impl GrpcReplyServer,
) -> Result<(), Box<dyn std::error::Error>>
where
    A: AmqClient + 'static,
    I: InteractionRepository + 'static,
    RS: ReplyService + 'static,
    PS: PostInteractionsService + 'static,
{
    let axum_address = SocketAddr::from(([0, 0, 0, 0], app_state.config.global.server_port));
    let axum_app = create_router(app_state.clone()).await;

    let grpc_address = app_state.config.grpc_server.address.parse()?;

    info!("Starting gRPC server on {}", grpc_address);
    let grpc_server_future = Server::builder()
        .layer(
            tower_http::trace::TraceLayer::new_for_grpc()
                .make_span_with({
                    let (address, port) = app_state
                        .config
                        .grpc_server
                        .address
                        .split_once(":")
                        .map(|(address, port)| (String::from(address), String::from(port)))
                        .unwrap();

                    move |request: &Request<_>| {
                        let request_id = request.headers().get(REQUEST_ID_HEADER);
                        let request_path = request.uri().path();
                        let http_method = request.method().to_string();

                        let (service, method) = request_path
                            .strip_prefix('/')
                            .unwrap()
                            .split_once('/')
                            .unwrap();
                        let parent_cx = global::get_text_map_propagator(|propagator| {
                            propagator.extract(&HeaderExtractor(request.headers()))
                        });

                        let span = info_span!(
                            "",
                            "otel.name" = format!("{} {}", &http_method, &request_path),
                            "rpc.system" = "grpc",
                            "server.address" = address,
                            "server.port" = port,
                            "rpc.method" = method,
                            "rpc.service" = service,
                            "otel.kind" = "server",
                            "http.request.id" = field::Empty,
                        );
                        span.set_parent(parent_cx);

                        if let Some(request_id) = request_id {
                            span.record("http.request.id", request_id.to_str().unwrap_or_default());
                        }

                        span
                    }
                })
                .on_request(DefaultOnRequest::new().level(tracing::Level::INFO))
                .on_response(
                    trace::DefaultOnResponse::new()
                        .level(tracing::Level::INFO)
                        .include_headers(true),
                )
                .on_failure(trace::DefaultOnFailure::new().level(tracing::Level::ERROR)),
        )
        .add_service(GrpcReplyServiceServer::new(grpc_server))
        .serve(grpc_address);

    info!("Starting Axum HTTP API server on {}", axum_address);
    let axum_server_future = axum::serve(TcpListener::bind(axum_address).await?, axum_app)
        .with_graceful_shutdown(shutdown_signal(app_state.clone()));

    tokio::select! {
        result = grpc_server_future => {
            if let Err(e) = result {
                error!("gRPC server error: {:?}", e);
            }
        }
        result = axum_server_future => {
            if let Err(e) = result {
                error!("Axum server error: {:?}", e);
            }
        }
    }

    info!("Servers stopped.");
    Ok(())
}

async fn shutdown_signal<A, I, RS, PS>(app_state: AppState<A, I, RS, PS>)
where
    A: AmqClient + 'static,
    I: InteractionRepository + 'static,
    RS: ReplyService + 'static,
    PS: PostInteractionsService + 'static,
{
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");

        tracing::info!("Stopping api application ...");
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{signal, SignalKind};
        let mut sigterm =
            signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");
        sigterm.recv().await;
        println!("SIGTERM received!");
    };

    #[cfg(unix)]
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    #[cfg(not(unix))]
    ctrl_c.await;

    app_state
        .close()
        .await
        .map_err(|e| tracing::error!("{:?}", e))
        .expect("Application could not properly shutdown!");

    tracing::info!("Shutdown complete.");
}
