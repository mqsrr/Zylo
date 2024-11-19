use std::sync::Arc;
use crate::models::app_state::AppState;
use crate::routes::post;
use crate::services::s3::S3Service;
use crate::services::{amq, database, redis};
use crate::settings::AppConfig;
use axum::http::header;
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::propagate_header::PropagateHeaderLayer;
use tower_http::sensitive_headers::SetSensitiveHeadersLayer;
use tower_http::trace;
use crate::services::user_profile::user_profile::user_profile_service_client::UserProfileServiceClient;
use crate::services::user_profile::UserProfileService;

pub async fn create_app<S: S3Service + Clone + Send + Sync + 'static>(
    config: AppConfig,
    s3file_service: Arc<S>,
) -> Router {
    let max_logging_level = map_log_level(&config.logger.level).unwrap_or(tracing::Level::INFO);
    tracing_subscriber::fmt::fmt()
        .with_max_level(max_logging_level)
        .pretty()
        .init();

    let db = database::init_db(&config.database).await;
    let redis = redis::create_client(&config.redis).await;
    let amq = amq::open_amq_connection(&config.amq).await;
    let user_profile = UserProfileService::new(redis.clone(), UserProfileServiceClient::connect(config.grpc_server.uri.clone()).await.unwrap());

    amq::consume_user_created(&amq, db.clone())
        .await
        .expect("Error configuring user created consumer");
    
    amq::consume_user_deleted(&amq, db.clone(), s3file_service.clone())
        .await
        .expect("Error configuring user deleted consumer");
    
    amq::consume_user_updated(&amq, db.clone(), redis.clone())
        .await
        .expect("Error configuring user updated consumer");

    let app_state = AppState {
        db,
        redis,
        user_profile_service: user_profile,
        s3file_service,
        amq,
        config,
    };

    Router::new()
        .merge(post::create_router(app_state))
        .layer(
            trace::TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().include_headers(true))
                .on_response(trace::DefaultOnResponse::new().level(tracing::Level::INFO)),
        )
        .layer(SetSensitiveHeadersLayer::new(std::iter::once(
            header::AUTHORIZATION,
        )))
        .layer(PropagateHeaderLayer::new(header::HeaderName::from_static(
            "x-request-id",
        )))
        .layer(CorsLayer::permissive())
}

fn map_log_level(level: &String) -> Option<tracing::Level> {
    match level.trim().to_lowercase().as_str() {
        "trace" => Some(tracing::Level::TRACE),
        "debug" => Some(tracing::Level::DEBUG),
        "info" => Some(tracing::Level::INFO),
        "warning" => Some(tracing::Level::WARN),
        "error" => Some(tracing::Level::ERROR),
        _ => None,
    }
}
