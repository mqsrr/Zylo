use std::net::SocketAddr;
use dotenv::dotenv;
use log::{info};
use tokio::net::TcpListener;
use tokio::signal;
use crate::models::app_state::AppState;
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
async fn main() {
    dotenv().ok();

    let key_vault = KeyVault::new().await.unwrap_or_else(|e| panic!("{}", e));
    let config = AppConfig::new(&key_vault).await;
    let app_state = AppState::new(&config).await;

    let address = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    let app = app::create_app(config, app_state.clone()).await;

    let listener = TcpListener::bind(address).await.unwrap();
    info!("Listening on: {}", address);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(app_state))
        .await
        .expect("Server could not start");
}


async fn shutdown_signal(app_state: AppState) {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
        println!("Ctrl+C received!");
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{signal, SignalKind};
        let mut sigterm = signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");
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

    app_state.close().await;
    println!("Shutdown complete.");
}