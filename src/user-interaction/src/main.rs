use crate::app::{init_logs, init_metrics, init_traces, init_tracing, run_app};
use crate::decorators::cache_service_decorator::DecoratedCacheService;
use crate::decorators::grpc_server_decorator::DecoratedGrpcServer;
use crate::decorators::posts_repo_decorator::DecoratedPostsRepository;
use crate::decorators::reply_repo_decorator::DecoratedReplyRepository;
use crate::decorators::users_repo_decorator::DecoratedUsersRepository;
use crate::models::app_state::AppState;
use crate::repositories::init_db;
use crate::repositories::interaction_repo::RedisInteractionRepository;
use crate::services::amq_client::{AmqClient, RabbitMqClient};
use crate::services::post_interactions_service::PostInteractionsServiceImpl;
use crate::services::reply_service::ReplyServiceImpl;
use crate::settings::AppConfig;
use dotenv::dotenv;
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
    init_tracing(&logger_provider, &trace_provider);

    let pg_pool = init_db(&config.database).await?;
    let reply_repo = Arc::new(
        DecoratedReplyRepository::new(pg_pool.clone())
            .observable()
            .build(),
    );

    let posts_repo = Arc::new(
        DecoratedPostsRepository::new(pg_pool.clone())
            .observable()
            .build(),
    );

    let users_repo = Arc::new(
        DecoratedUsersRepository::new(pg_pool.clone())
            .observable()
            .build(),
    );

    let cache_service = Arc::new(
        DecoratedCacheService::new(config.redis.clone())?
            .observable()
            .build(),
    );

    let interaction_repo = Arc::new(RedisInteractionRepository::new(cache_service.clone()));
    
    let reply_service = Arc::new(ReplyServiceImpl::new(
        reply_repo.clone(),
        interaction_repo.clone(),
    ));

    let post_interactions_service = Arc::new(PostInteractionsServiceImpl::new(
        reply_service.clone(),
        interaction_repo.clone(),
    ));

    let amq_client = Arc::new(RabbitMqClient::new(&config.amq).await?);
    amq_client
        .setup_listeners(
            posts_repo.clone(),
            users_repo.clone(),
            interaction_repo.clone(),
        )
        .await?;

    let grpc_server =
        DecoratedGrpcServer::new(reply_service.clone(), post_interactions_service.clone())
            .observable()
            .build();

    let app = AppState::new(
        amq_client,
        interaction_repo,
        reply_service,
        post_interactions_service,
        config,
    );

    run_app(app, grpc_server).await?;
    trace_provider.shutdown()?;
    meter_provider.shutdown()?;
    logger_provider.shutdown()?;
    Ok(())
}
