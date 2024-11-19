use crate::services::key_vault::KeyVault;
use crate::services::s3;
use crate::settings::AppConfig;
use dotenv::dotenv;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

mod app;
mod auth;
mod errors;
mod models;
mod routes;
mod services;
mod settings;
mod utils;

#[cfg(test)]
mod tests;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let key_vault = KeyVault::new().await.unwrap_or_else(|e| panic!("{:?}", e));
    let config = AppConfig::new(&key_vault).await;

    let s3_client = s3::init_s3_client().await.unwrap();
    let s3_service = Arc::new(s3::S3FileService::new(s3_client, config.s3_config.clone()));

    let address = SocketAddr::from(([127, 0, 0, 1], config.server.port));
    let app = app::create_app(config, s3_service).await;

    let listener = TcpListener::bind(address).await.unwrap();
    info!("Listening on: {}", &address);
    axum::serve(listener, app)
        .await
        .expect("Server could not start");
}
