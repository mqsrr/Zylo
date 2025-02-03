use crate::errors;
use crate::models::event_messages::{UserCreatedMessage, UserDeletedMessage};
use crate::repositories::post_repo::PostRepository;
use crate::repositories::user_repo::UsersRepository;
use crate::settings::RabbitMq;
use crate::utils::constants::{POST_EXCHANGE_NAME, USER_EXCHANGE_NAME};
use crate::utils::helpers::Finalizer;
use async_trait::async_trait;
use futures_util::Future;
use futures_util::StreamExt;
use lapin::options::{
    BasicAckOptions, BasicConsumeOptions, BasicNackOptions, BasicPublishOptions,
    ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions,
};
use lapin::types::FieldTable;
use lapin::{BasicProperties, Channel, Connection, ConnectionProperties};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::log::{error, info};

#[async_trait]
pub trait AmqClient: Send + Sync + Finalizer {
    async fn declare_exchanges(&self) -> Result<(), errors::AmqError>;
    async fn declare_queues(&self) -> Result<(), errors::AmqError>;

    async fn publish_event<T: Serialize + Send + Sync>(
        &self,
        exchange_name: &str,
        routing_key: &str,
        event: &T,
    ) -> Result<(), errors::AmqError>;

    async fn setup_listeners<U: UsersRepository + 'static, P: PostRepository + 'static>(
        &self,
        user_repo: Arc<U>,
        post_repo: Arc<P>,
    ) -> Result<(), errors::AppError>;
}

#[async_trait]
pub trait AmqConsumer: Send + Sync {
    async fn consume_user_created<U: UsersRepository + 'static>(
        &self,
        user_repo: Arc<U>,
    ) -> Result<(), errors::AppError>;

    async fn consume_user_deleted<U: UsersRepository + 'static, P: PostRepository + 'static>(
        &self,
        user_repo: Arc<U>,
        post_repo: Arc<P>,
    ) -> Result<(), errors::AppError>;
}

pub struct RabbitMqClient {
    connection: Connection,
    consumer_channels: Arc<Mutex<Vec<Channel>>>,
    publish_channel: Arc<Channel>,
}

impl RabbitMqClient {
    pub async fn new(config: &RabbitMq) -> Result<Self, errors::AmqError> {
        let connection = Connection::connect(&config.uri, ConnectionProperties::default()).await?;
        let publish_channel = connection.create_channel().await?;

        Ok(Self {
            connection,
            consumer_channels: Arc::new(Mutex::new(Vec::new())),
            publish_channel: Arc::new(publish_channel),
        })
    }

    pub async fn new_channel(&self) -> Result<Channel, errors::AmqError> {
        let channel = self.connection.create_channel().await?;
        self.consumer_channels.lock().await.push(channel.clone());

        Ok(channel)
    }

    async fn consume_event<T, F, Fut>(
        &self,
        queue_name: String,
        handler: F,
    ) -> Result<(), errors::AppError>
    where
        T: for<'de> serde::Deserialize<'de> + Send + 'static,
        F: Fn(T) -> Fut + Send + 'static + Clone,
        Fut: Future<Output = Result<(), errors::AppError>> + Send + 'static,
    {
        let channel = self.new_channel().await?;
        let mut consumer = channel
            .basic_consume(
                &queue_name,
                &format!("{}-consumer", queue_name),
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
            .map_err(errors::AmqError::ConnectionError)?;

        tokio::spawn(async move {
            while let Some(delivery_result) = consumer.next().await {
                match delivery_result {
                    Ok(delivery) => {
                        let event: T = match serde_json::from_slice(&delivery.data) {
                            Ok(event) => event,
                            Err(err) => {
                                error!(
                                    "Failed to deserialize message from {}: {}",
                                    queue_name, err
                                );

                                let options = BasicNackOptions {
                                    requeue: false,
                                    ..Default::default()
                                };
                                if let Err(err) = delivery.nack(options).await {
                                    error!("Failed to nack message from {}: {}", queue_name, err);
                                }

                                continue;
                            }
                        };

                        if let Err(err) = handler(event).await {
                            error!("{:?}", err);
                        }

                        if let Err(err) = delivery.ack(BasicAckOptions::default()).await {
                            error!(
                                "Failed to acknowledge message from {}: {:?}",
                                queue_name, err
                            );
                        }
                    }
                    Err(err) => {
                        error!("Failed to consume message from {}: {:?}", queue_name, err);
                    }
                }
            }
        });

        Ok(())
    }

    async fn handle_user_created<U: UsersRepository + 'static>(
        event: UserCreatedMessage,
        user_repo: Arc<U>,
    ) -> Result<(), errors::AppError> {
        user_repo.create(event.id).await
    }

    async fn handle_user_deleted<U: UsersRepository + 'static, P: PostRepository + 'static>(
        event: UserDeletedMessage,
        user_repo: Arc<U>,
        post_repo: Arc<P>,
    ) -> Result<(), errors::AppError> {
        let user_id = event.id;

        post_repo.delete_all_from_user(&user_id).await?;
        user_repo.delete(&user_id).await?;
    
        Ok(())
    }
}

#[async_trait]
impl AmqClient for RabbitMqClient {
    async fn declare_exchanges(&self) -> Result<(), errors::AmqError> {
        let exchange_options = ExchangeDeclareOptions {
            durable: true,
            ..Default::default()
        };

        self.publish_channel
            .exchange_declare(
                POST_EXCHANGE_NAME,
                lapin::ExchangeKind::Direct,
                exchange_options,
                FieldTable::default(),
            )
            .await?;

        self.publish_channel
            .exchange_declare(
                USER_EXCHANGE_NAME,
                lapin::ExchangeKind::Direct,
                exchange_options,
                FieldTable::default(),
            )
            .await?;

        Ok(())
    }

    async fn declare_queues(&self) -> Result<(), errors::AmqError> {
        let queue_options = QueueDeclareOptions {
            durable: true,
            ..Default::default()
        };
        let queue_map = vec![
            (
                "user-created-media-service-queue",
                USER_EXCHANGE_NAME,
                "user.created",
            ),
            (
                "user-deleted-media-service-queue",
                USER_EXCHANGE_NAME,
                "user.deleted",
            ),
        ];

        for (queue_name, exchange_name, routing_key) in queue_map {
            self.publish_channel
                .queue_declare(queue_name, queue_options, FieldTable::default())
                .await?;

            self.publish_channel
                .queue_bind(
                    queue_name,
                    exchange_name,
                    routing_key,
                    QueueBindOptions::default(),
                    FieldTable::default(),
                )
                .await?;
        }

        Ok(())
    }

    async fn publish_event<T: Serialize + Sync + Send>(
        &self,
        exchange_name: &str,
        routing_key: &str,
        event: &T,
    ) -> Result<(), errors::AmqError> {
        let message = serde_json::to_string(event).map_err(errors::AmqError::DeserializeError)?;
        self.publish_channel
            .basic_publish(
                exchange_name,
                routing_key,
                BasicPublishOptions::default(),
                message.as_bytes(),
                BasicProperties::default(),
            )
            .await?;

        Ok(())
    }

    async fn setup_listeners<U: UsersRepository + 'static, P: PostRepository + 'static>(
        &self,
        user_repo: Arc<U>,
        post_repo: Arc<P>,
    ) -> Result<(), errors::AppError> {
        self.declare_exchanges().await?;
        self.declare_queues().await?;

        self.consume_user_created(user_repo.clone()).await?;
        self.consume_user_deleted(user_repo, post_repo).await
    }
}

#[async_trait]
impl AmqConsumer for RabbitMqClient {
    async fn consume_user_created<U: UsersRepository + 'static>(
        &self,
        user_repo: Arc<U>,
    ) -> Result<(), errors::AppError> {
        self.consume_event(
            "user-created-media-service-queue".to_string(),
            move |event: UserCreatedMessage| {
                Box::pin({
                    let user_repo = user_repo.clone();
                    async move { RabbitMqClient::handle_user_created(event, user_repo).await }
                })
            },
        )
        .await
    }

    async fn consume_user_deleted<U: UsersRepository + 'static, P: PostRepository + 'static>(
        &self,
        user_repo: Arc<U>,
        post_repo: Arc<P>,
    ) -> Result<(), errors::AppError> {
        self.consume_event(
            "user-deleted-user-interaction-queue".to_string(),
            move |event: UserDeletedMessage| {
                Box::pin({
                    let user_repo = user_repo.clone();
                    let post_repo = post_repo.clone();
                    async move {
                        RabbitMqClient::handle_user_deleted(
                            event,
                            user_repo,
                            post_repo,
                        )
                        .await
                    }
                })
            },
        )
        .await
    }
}

#[async_trait]
impl Finalizer for RabbitMqClient {
    async fn finalize(&self) -> Result<(), errors::AppError> {
        info!("Closing RabbitMQ client connection...");

        let mut channels = self.consumer_channels.lock().await;
        for channel in channels.drain(..) {
            if let Err(e) = channel.close(200, "Shutting down").await {
                error!("Failed to close consumer channel: {:?}", e);
            }
        }

        if let Err(e) = self.publish_channel.close(200, "Shutting down").await {
            error!("Failed to close publish channel: {:?}", e);
        }

        if let Err(e) = self.connection.close(200, "Shutting down").await {
            error!("Failed to close connection: {:?}", e);
        }

        info!("RabbitMQ client connection closed");
        Ok(())
    }
}
