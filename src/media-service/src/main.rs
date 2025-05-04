use crate::app::{init_logs, init_metrics, init_trace, init_traces, run_app};
use crate::decorators::cache_service_decorator::ObservableCacheService;
use crate::decorators::grpc_server_decorator::ObservablePostServer;
use crate::decorators::post_repo_decorator::DecoratedPostRepositoryBuilder;
use crate::decorators::s3_service_decorator::DecoratedS3ServiceBuilder;
use crate::decorators::user_repo_decorator::DecoratedUserRepository;
use crate::services::amq::{AmqClient, RabbitMqClient};
use crate::services::cache_service::RedisCacheService;
use crate::services::grpc_server::GrpcPostServer;
use crate::utils::helpers::init_db;
use dotenv::dotenv;
use models::app_state::AppState;
use settings::AppConfig;
use std::sync::Arc;

mod app;
mod auth;
mod decorators;
mod errors;
mod models;
mod repositories;
mod routes;
mod services;
mod settings;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let config = AppConfig::new().await;

    let trace_provider = init_traces(&config.otel_collector.address);
    let meter_provider = init_metrics(&config.otel_collector.address);
    let logger_provider = init_logs(&config.otel_collector.address);
    init_trace(&logger_provider, &trace_provider);

    let mongo_db = init_db(&config.database).await;
    let cache_service = Arc::new(ObservableCacheService::new(RedisCacheService::new(
        config.redis.clone(),
    )?));

    let s3_service = Arc::new(
        DecoratedS3ServiceBuilder::new(config.s3_config.clone())
            .await
            .observable()
            .cached(cache_service.clone())
            .build(),
    );

    let post_repo = Arc::new(
        DecoratedPostRepositoryBuilder::new(&mongo_db, s3_service.clone())
            .observable()
            .cached(cache_service.clone())
            .build(),
    );

    let user_repo = Arc::new(
        DecoratedUserRepository::new(mongo_db)
            .observable()
            .cached(cache_service.clone())
            .build(),
    );

    let grpc_server = ObservablePostServer::new(GrpcPostServer::new(post_repo.clone()));

    let amq_client = Arc::new(RabbitMqClient::new(&config.amq).await?);
    amq_client
        .setup_listeners(user_repo.clone(), post_repo.clone())
        .await?;

    let app_state = AppState::new(post_repo, user_repo, cache_service, amq_client, config).await;

    run_app(app_state, grpc_server).await?;
    trace_provider.shutdown()?;
    meter_provider.shutdown()?;
    logger_provider.shutdown()?;

    Ok(())
}
