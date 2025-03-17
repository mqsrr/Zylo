use crate::services::key_vault::KeyVault;
use crate::utils::constants::{EXPOSED_PORT, GRPC_SERVER_ADDR, JWT_AUDIENCE, JWT_ISSUER, JWT_SECRET, OTEL_COLLECTOR_ADDR, POSTGRES_CONNECTION_STRING, RABBITMQ_URL_SECRET, REDIS_CONNECTION_STRING, REDIS_EXPIRE};
use serde::Deserialize;
use std::fs;

#[derive(Debug, Clone, Deserialize)]
pub struct Server {
    pub port: u16,
}

impl Server {
    async fn from_key_vault(key_vault: &KeyVault) -> Self {
        Self{
            port: key_vault.get_secret(EXPOSED_PORT).await.unwrap().parse().unwrap(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Database {
    pub uri: String,
}

impl Database {
    async fn from_key_vault(key_vault: &KeyVault) -> Self {
        Self{
            uri: key_vault.get_secret(POSTGRES_CONNECTION_STRING).await.unwrap(),
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
        Self{
            secret: key_vault.get_secret(JWT_SECRET).await.unwrap(),
            issuer: key_vault.get_secret(JWT_ISSUER).await.unwrap(),
            audience: key_vault.get_secret(JWT_AUDIENCE).await.unwrap(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Redis {
    pub uri: String,
    pub expire_time: u32,
}

impl Redis {
    async fn from_key_vault(key_vault: &KeyVault) -> Self {
        Self{
            uri: key_vault.get_secret(REDIS_CONNECTION_STRING).await.unwrap(),
            expire_time: key_vault.get_secret(REDIS_EXPIRE).await.unwrap().parse().unwrap(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RabbitMq {
    pub uri: String,
}

impl RabbitMq {
    async fn from_key_vault(key_vault: &KeyVault) -> Self {
        Self{
            uri: key_vault.get_secret(RABBITMQ_URL_SECRET).await.unwrap(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct GrpcServer {
    pub address: String,
}

impl GrpcServer {
   async fn from_key_vault(key_vault: &KeyVault) -> Self {
       Self {
           address: key_vault.get_secret(GRPC_SERVER_ADDR).await.unwrap()
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
           address: key_vault.get_secret(OTEL_COLLECTOR_ADDR).await.unwrap(),
       }
   } 
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: Server,
    pub database: Database,
    pub redis: Redis,
    pub auth: Auth,
    pub amq: RabbitMq,
    pub grpc_server: GrpcServer,
    pub otel_collector: OtelCollector
}

impl AppConfig {
    pub async fn new() -> Self {
        match std::env::var("APP_ENV"){
            Ok(env) => {
                if env.eq_ignore_ascii_case("production") {
                    return AppConfig::from_key_vault(&KeyVault::new().await.unwrap()).await
                }
                
                AppConfig::from_json("./config/development.json").await
            }
            Err(_) => {
                AppConfig::from_json("./config/development.json").await
            }
        }
    }
    
    async fn from_key_vault(key_vault: &KeyVault) -> Self {
        Self {
            server: Server::from_key_vault(key_vault).await,
            database: Database::from_key_vault(key_vault).await,
            redis: Redis::from_key_vault(key_vault).await,
            auth: Auth::from_key_vault(key_vault).await,
            amq: RabbitMq::from_key_vault(key_vault).await,
            grpc_server: GrpcServer::from_key_vault(key_vault).await,
            otel_collector: OtelCollector::from_key_vault(key_vault).await,
        }
    }

    async fn from_json(file: &str) -> Self {
        let config_str = fs::read_to_string(file).expect("Failed to read config file");
        serde_json::from_str(&config_str).expect("Invalid JSON configuration")
    }
}