pub const OTEL_SERVICE_NAME: &str = "media-service";
pub const MONGO_URL_SECRET: &str = "Media-MongoDb--ConnectionString";
pub const REDIS_URL_SECRET: &str = "Media-Redis--ConnectionString";

pub const REDIS_EXPIRE: &str = "Zylo-Redis--Expire";
pub const RABBITMQ_URL_SECRET: &str = "Zylo-RabbitMq--ConnectionString";

pub const GRPC_SERVER_ADDR: &str = "Media-gRPC--ServerAddr";
pub const OTEL_COLLECTOR_ADDR: &str = "Zylo-OTEL--CollectorAddress";

pub const S3_BUCKET_NAME: &str = "Zylo-S3--BucketName";
pub const S3_BUCKET_PRESIGNED_URL_EXPIRE_TIME: &str = "Zylo-S3--PresignedUrlExpire";

pub const POST_EXCHANGE_NAME: &str = "post-exchange";
pub const USER_EXCHANGE_NAME: &str = "user-exchange";

pub const JWT_SECRET: &str = "Zylo-Jwt--Secret";
pub const JWT_ISSUER: &str = "Zylo-Jwt--Issuer";
pub const JWT_AUDIENCE: &str = "Zylo-Jwt--Audience";
pub const REQUEST_ID_HEADER: &str = "x-request-id";