use crate::services::key_vault::KeyVault;
use crate::utils::constants::{JWT_AUDIENCE, JWT_ISSUER, JWT_SECRET, MONGO_URL_SECRET, RABBITMQ_URL_SECRET, REDIS_EXPIRE, REDIS_URL_SECRET, S3_BUCKET_NAME, S3_BUCKET_PRESIGNED_URL_EXPIRE_TIME, USER_GRPC_SERVER};
use serde::Deserialize;
use std::fmt;

#[derive(Debug, Clone, Deserialize)]
pub struct Server {
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Logger {
    pub level: String,
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
    pub uri: String,
}

impl GrpcServer {
    async fn from_key_vault(key_vault: &KeyVault) -> Self {
        Self {
            uri: key_vault.get_secret(USER_GRPC_SERVER).await.unwrap()
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: Server,
    pub logger: Logger,
    pub database: Database,
    pub redis: Redis,
    pub auth: Auth,
    pub amq: RabbitMq,
    pub s3_config: S3Settings,
    pub grpc_server: GrpcServer
}

impl AppConfig {
    pub async fn new(key_vault: &KeyVault) -> Self {
        Self {
            server: Server { port: 8082 },
            logger: Logger {
                level: "info".to_string(),
            },
            database: Database::from_key_vault(key_vault).await,
            redis: Redis::from_key_vault(key_vault).await,
            auth: Auth::from_key_vault(key_vault).await,
            amq: RabbitMq::from_key_vault(key_vault).await,
            s3_config: S3Settings::from_key_vault(key_vault).await,
            grpc_server: GrpcServer::from_key_vault(key_vault).await,
        }
    }
}

impl fmt::Display for Server {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "http://localhost:{}", &self.port)
    }
}
