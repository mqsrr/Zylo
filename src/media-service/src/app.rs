use crate::auth::authorization_middleware;
use crate::models::app_state::AppState;
use crate::repositories::post_repo::PostRepository;
use crate::repositories::user_repo::UsersRepository;
use crate::routes::post;
use crate::services::amq::AmqClient;
use crate::services::cache_service::CacheService;
use crate::services::grpc_server::post_server::post_service_server::{
    PostService, PostServiceServer,
};
use crate::utils::constants::REQUEST_ID_HEADER;
use axum::http::{header, HeaderName, Request};
use axum::{middleware, Router};
use opentelemetry::trace::TracerProvider as _;
use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::trace::TracerProvider;
use opentelemetry_sdk::Resource;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::signal;
use tonic::transport::Server;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::sensitive_headers::SetSensitiveHeadersLayer;
use tower_http::trace;
use tracing::log::{error, info};
use tracing::{field, info_span};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter};

pub fn init_tracing() -> TracerProvider {
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint("http://localhost:4317")
        .build()
        .unwrap();

    let resource = Resource::new(vec![
        KeyValue::new("service.name", "media-service"),
        KeyValue::new("service.version", "1.0.0"),
    ]);

    let provider = TracerProvider::builder()
        .with_resource(resource.clone())
        .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
        .build();

    global::set_tracer_provider(provider.clone());
    provider
}

pub fn init_trace(provider: &TracerProvider) {
    let tracer = provider.tracer("tracing-jaeger");
    tracing_subscriber::registry()
        .with(OpenTelemetryLayer::new(tracer))
        .with(fmt::layer().pretty())
        .with(EnvFilter::from_default_env())
        .init();
}

pub fn init_meter() -> SdkMeterProvider {
    let resource = Resource::new(vec![
        KeyValue::new("service.name", "user-interaction"),
        KeyValue::new("service.version", "1.0.0"),
    ]);

    let exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_tonic()
        .with_endpoint("http://localhost:4317")
        .with_timeout(Duration::from_secs(3))
        .build()
        .unwrap();

    let reader = opentelemetry_sdk::metrics::PeriodicReader::builder(
        exporter,
        opentelemetry_sdk::runtime::Tokio,
    )
    .with_interval(Duration::from_secs(3))
    .with_timeout(Duration::from_secs(10))
    .build();

    let provider = SdkMeterProvider::builder()
        .with_reader(reader)
        .with_resource(resource)
        .build();

    global::set_meter_provider(provider.clone());
    provider
}

pub async fn create_router<P, U, C, A>(
    app_state: AppState<P, U, C, A>,
) -> Router
where
    P: PostRepository + 'static,
    U: UsersRepository + 'static,
    C: CacheService + 'static,
    A: AmqClient + 'static,
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
                    let server_port = app_state.config.server.port.to_string();
                    move |request: &Request<_>| {
                        let request_id = request.headers().get(REQUEST_ID_HEADER);
                        let request_uri = request.uri();
                        let request_path = request_uri.path();
                        let method = request.method().to_string();
                        let span = info_span!(
                            "",
                            "otel.name" = format!("{} {}", &method, &request_path),
                            "http.request.method" = &method,
                            "http.request.method_original" = &method,
                            "server.address" = "0.0.0.0",
                            "server.port" = server_port,
                            "url.full" = request_uri.to_string(),
                            "otel.kind" = "server",
                            "http.request.id" = field::Empty,
                            "url.query" = field::Empty,
                            "http.response.status_code" = field::Empty,
                            "http.response.content_length" = field::Empty
                        );

                        if let Some(query) = request_uri.query() {
                            span.record("url.query", query);
                        }

                        if let Some(request_id) = request_id {
                            span.record("http.request.id", request_id.to_str().unwrap_or_default());
                        }

                        span
                    }
                })
                .on_request(trace::DefaultOnRequest::new().level(tracing::Level::INFO))
                .on_response(
                    trace::DefaultOnResponse::new()
                        .level(tracing::Level::INFO)
                        .include_headers(true),
                )
                .on_failure(trace::DefaultOnFailure::new().level(tracing::Level::ERROR)),
        )
        .layer(PropagateRequestIdLayer::new(x_request_id));

    Router::new()
        .merge(post::create_router(app_state.clone()))
        .layer(middleware)
        .layer(middleware::from_fn_with_state(
            app_state.config.auth.clone(),
            authorization_middleware,
        ))
        .layer(SetSensitiveHeadersLayer::new(std::iter::once(
            header::AUTHORIZATION,
        )))
        .layer(CorsLayer::permissive())
}

pub async fn run_app<P, U, C, A>(
    app_state: AppState<P, U, C, A>,
    grpc_server: impl PostService,
) -> Result<(), Box<dyn std::error::Error>>
where
    P: PostRepository + 'static,
    U: UsersRepository + 'static,
    C: CacheService + 'static,
    A: AmqClient + 'static,
{
    let axum_address = SocketAddr::from(([0, 0, 0, 0], app_state.config.server.port));
    let axum_app = create_router(app_state.clone()).await;

    let grpc_address = format!("[::1]:{}", app_state.config.grpc_server.port)
        .parse()
        .unwrap();

    info!("Starting gRPC server on {}", grpc_address);
    let grpc_server_future = Server::builder()
        .add_service(PostServiceServer::new(grpc_server))
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

async fn shutdown_signal<P, U, C, A>(app_state: AppState<P, U, C, A>)
where
    P: PostRepository + 'static,
    U: UsersRepository + 'static,
    C: CacheService + 'static,
    A: AmqClient + 'static,
{
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");

        tracing::info!("Received Ctrl+C signal. Initiating API shutdown...");
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{signal, SignalKind};
        let mut sigterm =
            signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");
        sigterm.recv().await;
        tracing::info!("SIGTERM signal received. Initiating API shutdown...");
    };

    #[cfg(unix)]
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    #[cfg(not(unix))]
    ctrl_c.await;
    tracing::info!("Finalizing application shutdown. Cleaning up resources...");

    app_state
        .close()
        .await
        .map_err(|e| tracing::error!("Error during shutdown: {:?}", e))
        .expect("Failed to cleanly shut down the application.");

    tracing::info!("API shutdown completed successfully. Goodbye!");
}
