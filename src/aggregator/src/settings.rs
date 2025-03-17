use crate::services::key_vault::KeyVault;
use crate::utils::constants::{
    EXPOSED_PORT, FEED_SERVICE_GRPC_SERVER_ADDRESS, JWT_AUDIENCE, JWT_ISSUER, JWT_SECRET,
    MEDIA_SERVICE_GRPC_SERVER_ADDRESS, OTEL_COLLECTOR_ADDRESS, SOCIAL_GRAPH_GRPC_SERVER_ADDRESS,
    USER_INTERACTION_GRPC_SERVER_ADDRESS, USER_MANAGEMENT_GRPC_SERVER_ADDRESS,
};
use serde::Deserialize;
use std::fs;

#[derive(Debug, Clone, Deserialize)]
pub struct Server {
    pub port: u16,
}

impl Server {
    async fn from_key_vault(key_vault: &KeyVault) -> Self {
        Self {
            port: key_vault
                .get_secret(EXPOSED_PORT)
                .await
                .unwrap()
                .parse()
                .unwrap(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Auth {
    pub secret: String,
    pub issuer: String,
    pub audience: String,
}

impl Auth {
    async fn from_key_vault(key_vault: &KeyVault) -> Self {
        Self {
            secret: key_vault.get_secret(JWT_SECRET).await.unwrap(),
            issuer: key_vault.get_secret(JWT_ISSUER).await.unwrap(),
            audience: key_vault.get_secret(JWT_AUDIENCE).await.unwrap(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExternalGrpcServers {
    pub user_management: String,
    pub media_service: String,
    pub user_interactions_service: String,
    pub social_graph: String,
    pub feed_service: String,
}

impl ExternalGrpcServers {
    async fn from_key_vault(key_vault: &KeyVault) -> Self {
        Self {
            user_management: key_vault
                .get_secret(USER_MANAGEMENT_GRPC_SERVER_ADDRESS)
                .await
                .unwrap(),
            media_service: key_vault
                .get_secret(MEDIA_SERVICE_GRPC_SERVER_ADDRESS)
                .await
                .unwrap(),
            user_interactions_service: key_vault
                .get_secret(USER_INTERACTION_GRPC_SERVER_ADDRESS)
                .await
                .unwrap(),
            social_graph: key_vault
                .get_secret(SOCIAL_GRAPH_GRPC_SERVER_ADDRESS)
                .await
                .unwrap(),
            feed_service: key_vault
                .get_secret(FEED_SERVICE_GRPC_SERVER_ADDRESS)
                .await
                .unwrap(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct OtelCollector {
    pub address: String,
}

impl OtelCollector {
    async fn from_key_vault(key_vault: &KeyVault) -> Self {
        Self {
            address: key_vault.get_secret(OTEL_COLLECTOR_ADDRESS).await.unwrap(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: Server,
    pub auth: Auth,
    pub external_grpc_servers: ExternalGrpcServers,
    pub otel_collector: OtelCollector,
}

impl AppConfig {
    pub async fn new() -> Self {
        match std::env::var("APP_ENV") {
            Ok(env) => {
                if env.eq_ignore_ascii_case("production") {
                    return AppConfig::from_key_vault(&KeyVault::new().await.unwrap()).await;
                }

                AppConfig::from_json("./config/development.json").await
            }
            Err(_) => AppConfig::from_json("./config/development.json").await,
        }
    }

    async fn from_key_vault(key_vault: &KeyVault) -> Self {
        Self {
            server: Server::from_key_vault(key_vault).await,
            auth: Auth::from_key_vault(key_vault).await,
            external_grpc_servers: ExternalGrpcServers::from_key_vault(key_vault).await,
            otel_collector: OtelCollector::from_key_vault(key_vault).await,
        }
    }

    async fn from_json(file: &str) -> Self {
        let config_str = fs::read_to_string(file).expect("Failed to read config file");
        serde_json::from_str(&config_str).expect("Invalid JSON configuration")
    }
}
