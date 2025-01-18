use crate::auth::authorization_middleware;
use crate::models::app_state::AppState;
use crate::observability::metrics::metrics_handler;
use crate::repositories::interaction_repo::InteractionRepository;
use crate::repositories::reply_repo::ReplyRepository;
use crate::routes;
use crate::services::amq_client::AmqClient;
use crate::services::cache_service::CacheService;
use crate::services::grpc_server::reply_server;
use crate::services::grpc_server::reply_server::grpc_reply_service_server::{
    GrpcReplyService, GrpcReplyServiceServer,
};
use crate::utils::constants::REQUEST_ID_HEADER;
use axum::http::{header, HeaderName, Request};
use axum::routing::get;
use axum::{middleware, Router};
use prometheus::Registry;
use std::net::SocketAddr;
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

pub async fn create_router<
    R: ReplyRepository + 'static,
    I: InteractionRepository + 'static,
    A: AmqClient + 'static,
    C: CacheService + 'static,
>(
    app_state: AppState<R, I, A, C>,
    registry: Registry,
) -> Router {
    let x_request_id = HeaderName::from_static(REQUEST_ID_HEADER);
    let middleware = ServiceBuilder::new()
        .layer(SetRequestIdLayer::new(
            x_request_id.clone(),
            MakeRequestUuid,
        ))
        .layer(
            trace::TraceLayer::new_for_http()
                .make_span_with(|request: &Request<_>| {
                    let request_id = request.headers().get(REQUEST_ID_HEADER);
                    let request_uri = request.uri();
                    let span = info_span!(
                        "",
                        "otel.name" = format!(
                            "{method} {target}",
                            method = request.method().as_str(),
                            target = request_uri.path()
                        ),
                        "otel.kind" = "server",
                        "http.request.id" = field::Empty,
                        "url.query" = field::Empty
                    );

                    if let Some(query) = request_uri.query() {
                        span.record("url.query", query);
                    }

                    if let Some(request_id) = request_id {
                        span.record("http.request.id", request_id.to_str().unwrap_or_default());
                    }

                    span
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
        .merge(routes::reply::create_router(app_state.clone()))
        .merge(routes::interaction::create_router(app_state.clone()))
        .route("/metrics", get(move || metrics_handler(registry)))
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

pub async fn run_app<
    R: ReplyRepository + 'static,
    I: InteractionRepository + 'static,
    A: AmqClient + 'static,
    C: CacheService + 'static,
>(
    app_state: AppState<R, I, A, C>,
    grpc_server: impl GrpcReplyService,
    registry: Registry,
) -> Result<(), Box<dyn std::error::Error>> {
    let axum_address = SocketAddr::from(([0, 0, 0, 0], app_state.config.server.port));
    let axum_app = create_router(app_state.clone(), registry).await;

    let grpc_address = format!("[::1]:{}", app_state.config.grpc_server.port)
        .parse()
        .unwrap();

    info!("Starting gRPC server on {}", grpc_address);
    let grpc_service_reflection = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(reply_server::FILE_DESCRIPTOR_SET)
        .build_v1()
        .unwrap();

    let grpc_server_future = Server::builder()
        .add_service(GrpcReplyServiceServer::new(grpc_server))
        .add_service(grpc_service_reflection)
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

async fn shutdown_signal<
    R: ReplyRepository + 'static,
    I: InteractionRepository + 'static,
    A: AmqClient + 'static,
    C: CacheService + 'static,
>(
    app_state: AppState<R, I, A, C>,
) {
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
