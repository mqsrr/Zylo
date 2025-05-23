use crate::services::key_vault::KeyVault;
use crate::utils::constants::{GRPC_SERVER_ADDR, JWT_AUDIENCE, JWT_ISSUER, JWT_SECRET, MONGO_URL_SECRET, OTEL_COLLECTOR_ADDR, RABBITMQ_URL_SECRET, REDIS_EXPIRE, REDIS_URL_SECRET, S3_BUCKET_NAME, S3_BUCKET_PRESIGNED_URL_EXPIRE_TIME};
use serde::Deserialize;
use std::fs;

#[derive(Debug, Clone, Deserialize)]
pub struct Global {
    pub server_port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Database {
    pub uri: String,
    pub name: String,
}

impl Database {
    async fn from_key_vault(key_vault: &KeyVault) -> Self {
        Self {
            uri: key_vault.get_secret(MONGO_URL_SECRET).await.unwrap(),
            name: "posts".to_string(),
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
pub struct Redis {
    pub uri: String,
    pub expire_time: u32,
}

impl Redis {
    async fn from_key_vault(key_vault: &KeyVault) -> Self {
        Self {
            uri: key_vault.get_secret(REDIS_URL_SECRET).await.unwrap(),
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
        Self {
            uri: key_vault.get_secret(RABBITMQ_URL_SECRET).await.unwrap(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct S3Settings {
    pub bucket_name: String,
    pub expire_time: u32,
}

impl S3Settings {
    async fn from_key_vault(key_vault: &KeyVault) -> Self {
        Self {
            bucket_name: key_vault.get_secret(S3_BUCKET_NAME).await.unwrap(),
            expire_time: key_vault
                .get_secret(S3_BUCKET_PRESIGNED_URL_EXPIRE_TIME)
                .await
                .unwrap()
                .parse()
                .unwrap(),
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
            address: key_vault.get_secret(GRPC_SERVER_ADDR).await.unwrap(),
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
    pub global: Global,
    pub database: Database,
    pub redis: Redis,
    pub auth: Auth,
    pub amq: RabbitMq,
    pub s3_config: S3Settings,
    pub grpc_server: GrpcServer,
    pub otel_collector: OtelCollector
}

impl AppConfig {
    pub async fn new() -> Self {
        match std::env::var("APP_ENV"){
            Ok(env) => {
                if env.eq_ignore_ascii_case("production"){
                    return AppConfig::from_key_vault(&KeyVault::new().await.unwrap()).await
                }

                let mut config = AppConfig::from_json("./config/development.json").await;
                config.s3_config.bucket_name = std::env::var("S3_BUCKET").unwrap();
                
                config
            }
            Err(_) => {
                AppConfig::from_json("./config/development.json").await
            }
        }
    }

    async fn from_key_vault(key_vault: &KeyVault) -> Self {
        let global = Global{
            server_port: std::env::var("SERVER_PORT").unwrap_or_default().parse::<u16>().unwrap_or(8080),
        };

        Self {
            global,
            database: Database::from_key_vault(key_vault).await,
            s3_config: S3Settings::from_key_vault(key_vault).await,
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