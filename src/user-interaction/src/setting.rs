use crate::services::key_vault::KeyVault;
use crate::utils::constants::{JWT_AUDIENCE, JWT_ISSUER, JWT_SECRET, POSTGRES_CONNECTION_STRING, RABBITMQ_URL_SECRET, REDIS_BACKUP_CONNECTION_STRING, REDIS_CONNECTION_STRING, REDIS_EXPIRE, USER_GRPC_SERVER};
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
    pub backup_uri: String,
    pub expire_time: u32,
}

impl Redis {
    async fn from_key_vault(key_vault: &KeyVault) -> Self {
        Self{
            uri: key_vault.get_secret(REDIS_CONNECTION_STRING).await.unwrap(),
            backup_uri: key_vault.get_secret(REDIS_BACKUP_CONNECTION_STRING).await.unwrap(),
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
    pub grpc_server: GrpcServer,
}

impl AppConfig {
    pub async fn new (key_vault: &KeyVault) -> Self {
        Self{
            server: Server{ port: 8083 },
            logger: Logger {level: "info".to_string()},
            database: Database::from_key_vault(key_vault).await,
            redis: Redis::from_key_vault(key_vault).await,
            auth: Auth::from_key_vault(key_vault).await,
            amq: RabbitMq::from_key_vault(key_vault).await,
            grpc_server: GrpcServer::from_key_vault(key_vault).await,
        }
    }
}

impl fmt::Display for Server {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "http://localhost:{}", &self.port)
    }
}