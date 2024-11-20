﻿pub const POSTGRES_CONNECTION_STRING: &str= "UserInteractions-Postgres--ConnectionString";
pub const REDIS_CONNECTION_STRING: &str= "UserInteractions-Redis--ConnectionString";
pub const REDIS_BACKUP_CONNECTION_STRING: &str= "UserInteractions-BackupRedis--ConnectionString";
pub const REDIS_EXPIRE: &str= "Zylo-Redis--Expire";

pub const EXPOSED_PORT: &str= "UserInteraction-API--ExposedPort";

pub const USER_GRPC_SERVER: &str = "UserManagement-Grpc--ServerAddress";

pub const RABBITMQ_URL_SECRET: &str = "Zylo-RabbitMq--ConnectionString";

pub const POST_EXCHANGE_NAME: &str = "post-exchange";
pub const USER_EXCHANGE_NAME: &str = "user-exchange";

pub const JWT_SECRET: &str = "Zylo-Jwt--Secret";
pub const JWT_ISSUER: &str = "Zylo-Jwt--Issuer";
pub const JWT_AUDIENCE: &str = "Zylo-Jwt--Audience";
