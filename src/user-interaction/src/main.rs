use crate::app::run_app;
use crate::models::app_state::AppState;
use crate::observability::metrics::init_prometheus;
use crate::observability::tracing::init_tracing;
use crate::observability::{
    ObservableCacheService, ObservableInteractionRepository, ObservableReplyRepository,
    ObservableReplyServer,
};
use crate::repositories::interaction_repo::RedisInteractionRepository;
use crate::repositories::reply_repo::PostgresReplyRepository;
use crate::services::amq_client::{AmqClient, RabbitMqClient};
use crate::services::backup_worker;
use crate::services::cache_service::RedisCacheService;
use crate::services::grpc_server::ReplyServer;
use crate::settings::AppConfig;
use dotenv::dotenv;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_sdk::trace::TracerProvider;
use std::sync::Arc;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter};

mod app;
mod auth;
mod errors;
mod models;
mod observability;
mod repositories;
mod routes;
mod services;
mod settings;
mod utils;

fn init_trace(provider: &TracerProvider) {
    let tracer = provider.tracer("tracing-jaeger");
    tracing_subscriber::registry()
        .with(OpenTelemetryLayer::new(tracer))
        .with(fmt::layer().pretty())
        .with(EnvFilter::from_default_env())
        .init();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let trace_provider = init_tracing();
    let registry = init_prometheus();

    init_trace(&trace_provider);

    let config = AppConfig::new().await;
    let reply_repo = PostgresReplyRepository::new(&config.database).await?;
    let reply_repo = Arc::new(ObservableReplyRepository::new(reply_repo, &registry)?);

    let cache_service = RedisCacheService::new(config.redis.clone())?;
    let cache_service = Arc::new(ObservableCacheService::new(cache_service, &registry)?);

    let interaction_repo = RedisInteractionRepository::new(cache_service.clone());
    let interaction_repo = Arc::new(ObservableInteractionRepository::new(
        interaction_repo,
        &registry,
    )?);

    let amq_client = Arc::new(RabbitMqClient::new(&config.amq).await?);
    amq_client
        .setup_listeners(
            reply_repo.clone(),
            interaction_repo.clone(),
            cache_service.clone(),
        )
        .await?;

    let grpc_server = ReplyServer::new(
        reply_repo.clone(),
        interaction_repo.clone(),
        cache_service.clone(),
    );
    let grpc_server = ObservableReplyServer::new(grpc_server, &registry)?;

    backup_worker::start_worker(config.redis.clone()).await?;
    let app = AppState::new(
        reply_repo,
        interaction_repo,
        amq_client,
        cache_service,
        config,
    );

    run_app(app, grpc_server, registry).await?;
    trace_provider.shutdown()?;

    Ok(())
}
