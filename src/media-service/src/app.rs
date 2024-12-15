use crate::models::app_state::AppState;
use crate::routes::post;
use crate::settings::AppConfig;
use axum::http::header;
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::propagate_header::PropagateHeaderLayer;
use tower_http::sensitive_headers::SetSensitiveHeadersLayer;
use tower_http::trace;

pub fn create_app(config: AppConfig, app_state: AppState) -> Router {
    let max_logging_level = map_log_level(&config.logger.level).unwrap_or(tracing::Level::INFO);
    tracing_subscriber::fmt::fmt()
        .with_max_level(max_logging_level)
        .pretty()
        .init();

    Router::new()
        .merge(post::create_router(app_state))
        .layer(
            trace::TraceLayer::new_for_http()
                .on_request(|request, _span| {
                    tracing::info!(
                        method = %request.method(),
                        uri = %request.uri(),
                        headers = ?request.headers()
                    );
                })
                .make_span_with(trace::DefaultMakeSpan::new().include_headers(true))
                .on_response(trace::DefaultOnResponse::new()
                    .level(trace::Level::INFO)
                    .include_headers(true)) 
                .on_failure(trace::DefaultOnFailure::new().level(trace::Level::ERROR)),
        )
        .layer(SetSensitiveHeadersLayer::new(std::iter::once(
            header::AUTHORIZATION,
        )))
        .layer(PropagateHeaderLayer::new(header::HeaderName::from_static(
            "x-request-id",
        )))
        .layer(CorsLayer::permissive())
}

fn map_log_level(level: &str) -> Option<tracing::Level> {
    match level.trim().to_lowercase().as_str() {
        "trace" => Some(tracing::Level::TRACE),
        "debug" => Some(tracing::Level::DEBUG),
        "info" => Some(tracing::Level::INFO),
        "warning" => Some(tracing::Level::WARN),
        "error" => Some(tracing::Level::ERROR),
        _ => None,
    }
}
