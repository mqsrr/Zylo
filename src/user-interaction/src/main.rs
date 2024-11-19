use std::net::SocketAddr;
use dotenv::dotenv;
use log::{info};
use tokio::net::TcpListener;
use crate::services::key_vault::KeyVault;
use crate::setting::AppConfig;

mod models;
mod app;
mod setting;
mod utils;
mod routes;
mod errors;
mod auth;
mod services;

#[tokio::main]
async fn main(){
    dotenv().ok();
    
    let key_vault = KeyVault::new().await.unwrap_or_else(|e|panic!("{}", e));
    let config = AppConfig::new(&key_vault).await;
    
    let address = SocketAddr::from(([127, 0, 0, 1], config.server.port));
    let app = app::create_app(config).await;
    
    let listener = TcpListener::bind(address).await.unwrap();
    info!("Listening on: {}", address);
    axum::serve(listener, app).await.expect("Server could not start");
}
