﻿pub const POSTGRES_CONNECTION_STRING: &str= "UserInteractions-Postgres--ConnectionString";
pub const REDIS_CONNECTION_STRING: &str= "UserInteractions-Redis--ConnectionString";
pub const REDIS_EXPIRE: &str= "Zylo-Redis--Expire";

pub const GRPC_SERVER_ADDR: &str = "UserInteraction-gRPC--ServerAddr";

pub const OTEL_SERVICE_NAME: &str = "user-interaction";
pub const OTEL_COLLECTOR_ADDR: &str = "Zylo-OTEL--CollectorAddress";

pub const RABBITMQ_URL_SECRET: &str = "Zylo-RabbitMq--ConnectionString";

pub const POST_EXCHANGE_NAME: &str = "post-exchange";
pub const USER_EXCHANGE_NAME: &str = "user-exchange";

pub const JWT_SECRET: &str = "Zylo-Jwt--Secret";
pub const JWT_ISSUER: &str = "Zylo-Jwt--Issuer";
pub const JWT_AUDIENCE: &str = "Zylo-Jwt--Audience";

pub const REQUEST_ID_HEADER: &str = "x-request-id";
