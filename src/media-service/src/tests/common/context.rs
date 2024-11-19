use crate::errors::AppError;
use crate::models::file::{File, FileMetadataResponse};
use crate::services::s3::S3Service;
use crate::settings::{AppConfig, Auth, Database, Logger, RabbitMq, Redis, S3Settings, Server};
use async_trait::async_trait;
use mockall::mock;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::future::Future;
use std::sync::Arc;
use test_context::AsyncTestContext;
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, ImageExt};
use testcontainers_modules::mongo::Mongo;
use testcontainers_modules::redis::RedisStack;


mock! {
        pub S3FileService {}
        
        #[async_trait]
        impl S3Service for S3FileService {
            async fn upload(&self, file: &File) -> Result<String, AppError>;
            async fn delete(&self, key: &str) -> Result<(), AppError>;
            async fn get_file_response(&self, key: &str) -> Result<FileMetadataResponse, AppError>;
            async fn get_presigned_url_for_download(&self, key: &str) -> Result<String, AppError>;
        }
    
        impl Clone for S3FileService {
            fn clone(&self) -> Self;
        }
    }


pub struct Context {
    pub redis_container: ContainerAsync<RedisStack>,
    pub mongo_container: ContainerAsync<Mongo>,
    pub rabbit_mq: ContainerAsync<testcontainers_modules::rabbitmq::RabbitMq>,
    pub s3_service: Arc<MockS3FileService>,
    pub app_config: AppConfig,
}

impl AsyncTestContext for Context {
    async fn setup() -> Context {
        let mongo = Mongo::default().start().await.unwrap();

        let redis = RedisStack::default().start().await.unwrap();
        let amq = testcontainers_modules::rabbitmq::RabbitMq::default().start().await.unwrap();

        let mongo_host = mongo.get_host().await.unwrap();
        let mongo_port = mongo.get_host_port_ipv4(27017).await.unwrap();

        let redis_host = redis.get_host().await.unwrap();
        let redis_port = redis.get_host_port_ipv4(6379).await.unwrap();

        let amq_host = amq.get_host().await.unwrap();
        let amq_port = amq.get_host_port_ipv4(5672).await.unwrap();

        let s3_service = Arc::new(MockS3FileService::new());
        let app_config = AppConfig {
            server: Server { port: 9000 },
            logger: Logger {
                level: "debug".to_string(),
            },
            database: Database {
                uri: format!("mongodb://{}:{}", mongo_host, mongo_port),
                name: String::from("media"),
            },
            redis: Redis {
                uri: format!("redis://{}:{}", redis_host, redis_port),
            },
            auth: Auth {
                secret: String::from("BeVerySaveWithThisVALLLUUUUEEE"),
                issuer: String::from("zylo-testing"),
                audience: String::from("zylo-testing"),
            },
            amq: RabbitMq {
                uri: format!("amqp://{}:{}", amq_host, amq_port),
            },
            s3_config: S3Settings {
                bucket_name: String::from("testing"),
                expire_time: 1,
            },
        };

        Context {
            redis_container: redis,
            mongo_container: mongo,
            rabbit_mq: amq,
            s3_service: s3_service.clone(),
            app_config: app_config.clone(),
        }
    }

    async fn teardown(self) {
        self.redis_container.stop().await.unwrap();
        self.mongo_container.stop().await.unwrap();
        self.rabbit_mq.stop().await.unwrap();
    }
}