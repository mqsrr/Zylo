use crate::app::{init_logs, init_metrics, init_trace, init_traces, run_app};
use crate::models::app_state::AppState;
use crate::services::aggregator::feed_service_client::FeedServiceClient;
use crate::services::aggregator::post_service_client::PostServiceClient;
use crate::services::aggregator::relationship_service_client::RelationshipServiceClient;
use crate::services::aggregator::reply_service_client::ReplyServiceClient;
use crate::services::aggregator::user_profile_service_client::UserProfileServiceClient;
use crate::services::feed_service::FeedServiceImpl;
use crate::services::post_service::PostsServiceImpl;
use crate::services::user_service::UserServiceImpl;
use crate::settings::AppConfig;
use dotenv::dotenv;
use std::sync::Arc;
use tokio::sync::Mutex;

mod app;
mod auth;
mod errors;
mod models;
mod routes;
mod services;
mod settings;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let config = AppConfig::new().await;
    let collector_address = &config.otel_collector.address;

    let trace_provider = init_traces(collector_address);
    let meter_provider = init_metrics(collector_address);
    let logger_provider = init_logs(collector_address);

    init_trace(&logger_provider, &trace_provider);

    let post_client =
        PostServiceClient::connect(config.external_grpc_servers.media_service.clone()).await?;

    let user_client =
        UserProfileServiceClient::connect(config.external_grpc_servers.user_management.clone())
            .await?;

    let relationship_client =
        RelationshipServiceClient::connect(config.external_grpc_servers.social_graph.clone())
            .await?;

    let reply_client = ReplyServiceClient::connect(
        config
            .external_grpc_servers
            .user_interactions_service
            .clone(),
    )
    .await?;
    let feed_client =
        FeedServiceClient::connect(config.external_grpc_servers.feed_service.clone()).await?;

    let posts_service = Arc::new(Mutex::new(PostsServiceImpl::new(
        post_client.clone(),
        reply_client.clone(),
        user_client.clone(),
    )));

    let users_service = Arc::new(Mutex::new(UserServiceImpl::new(
        user_client,
        relationship_client,
        post_client,
        reply_client,
    )));

    let feed_service = Arc::new(Mutex::new(FeedServiceImpl::new(
        feed_client,
        posts_service.clone(),
    )));

    let app_state = AppState::new(posts_service, users_service, feed_service, config);

    run_app(app_state).await?;

    trace_provider.shutdown()?;
    meter_provider.shutdown()?;
    logger_provider.shutdown()?;
    Ok(())
}
