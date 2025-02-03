use crate::app::{init_meter, init_trace, init_tracing, run_app};
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
    
    let trace_provider = init_tracing();
    let meter_provider = init_meter();
    
    init_trace(&trace_provider);
    let config = AppConfig::new().await;
    let mongo_db = init_db(&config.database).await;

    let cache_service = Arc::new(ObservableCacheService::new(RedisCacheService::new(
        config.redis.clone(),
    )?));

    let s3_service = Arc::new(
        DecoratedS3ServiceBuilder::new(config.s3_config.clone())
            .await
            .cached(cache_service.clone())
            .observable()
            .build(),
    );

    let post_repo = Arc::new(
        DecoratedPostRepositoryBuilder::new(&mongo_db, s3_service.clone())
            .cached(cache_service.clone())
            .observable()
            .build(),
    );

    let user_repo = Arc::new(
        DecoratedUserRepository::new(mongo_db)
            .cached(cache_service.clone())
            .observable()
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
    Ok(())
}
