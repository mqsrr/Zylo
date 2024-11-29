use crate::errors::AppError;
use crate::models::event_messages::{UserCreatedMessage, UserDeletedMessage, UserUpdatedMessage};
use crate::models::post::Post;
use crate::models::user::User;
use crate::services::redis;
use crate::settings::RabbitMq;
use crate::utils::constants::{POST_EXCHANGE_NAME, USER_EXCHANGE_NAME};
use futures_util::Future;
use futures_util::StreamExt;
use lapin::options::{
    BasicAckOptions, BasicConsumeOptions, BasicPublishOptions, ExchangeDeclareOptions,
    QueueBindOptions, QueueDeclareOptions,
};
use lapin::types::FieldTable;
use lapin::{BasicProperties, Channel, Connection, ConnectionProperties};
use log::warn;
use mongodb::Database;
use serde::Serialize;

use super::s3::S3FileService;

pub async fn open_amq_connection(config: &RabbitMq) -> Channel {
    let connection = Connection::connect(&config.uri, ConnectionProperties::default())
        .await
        .unwrap();
    let channel = connection.create_channel().await.unwrap();

    declare_exchanges(&channel).await.unwrap();
    declare_queues(&channel).await.unwrap();

    channel
}

async fn declare_exchanges(channel: &Channel) -> Result<(), AppError> {
    let exchange_options = ExchangeDeclareOptions {
        durable: true,
        ..Default::default()
    };

    channel
        .exchange_declare(
            POST_EXCHANGE_NAME,
            lapin::ExchangeKind::Direct,
            exchange_options,
            FieldTable::default(),
        )
        .await?;
    channel
        .exchange_declare(
            USER_EXCHANGE_NAME,
            lapin::ExchangeKind::Direct,
            exchange_options,
            FieldTable::default(),
        )
        .await?;

    Ok(())
}

async fn declare_queues(channel: &Channel) -> Result<(), AppError> {
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
            "user-updated-media-service-queue",
            USER_EXCHANGE_NAME,
            "user.updated",
        ),
        (
            "user-deleted-media-service-queue",
            USER_EXCHANGE_NAME,
            "user.deleted",
        ),
    ];

    for (queue_name, exchange_name, routing_key) in queue_map {
        channel
            .queue_declare(queue_name, queue_options, FieldTable::default())
            .await?;
        channel
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

pub async fn publish_event<T: Serialize>(
    channel: &Channel,
    exchange_name: &str,
    routing_key: &str,
    event: &T,
) -> Result<(), AppError> {
    let message = serde_json::to_string(event).unwrap();
    channel
        .basic_publish(
            exchange_name,
            routing_key,
            BasicPublishOptions::default(),
            message.as_bytes(),
            BasicProperties::default(),
        )
        .await?
        .await?;

    Ok(())
}

pub async fn consume_user_created(channel: &Channel, db: Database) -> Result<(), AppError> {
    consume_event(
        channel,
        "user-created-media-service-queue".to_string(),
        move |event: UserCreatedMessage| {
            let db = db.clone();
            Box::pin(async move { handle_user_created(event, db).await })
        },
    )
    .await
}

async fn handle_user_created(event: UserCreatedMessage, db: Database) -> Result<(), AppError> {
    User::create(event, &db).await?;
    Ok(())
}

pub async fn consume_user_deleted(
    channel: &Channel,
    db: Database,
    s3file_service: S3FileService,
) -> Result<(), AppError> {
    consume_event(
        channel,
        "user-deleted-media-service-queue".to_string(),
        move |event: UserDeletedMessage| {
            let db = db.clone();
            let s3file_service = s3file_service.clone();
            Box::pin(async move { handle_user_deleted(event, db, s3file_service).await })
        },
    )
    .await
}

async fn handle_user_deleted(
    event: UserDeletedMessage,
    db: Database,
    s3file_service: S3FileService,
) -> Result<(), AppError> {
    Post::delete_all_from_user(event.id, &db, &s3file_service).await?;
    User::delete(event, &db).await?;

    Ok(())
}

pub async fn consume_user_updated(
    channel: &Channel,
    db: Database,
    redis: ::redis::Client,
) -> Result<(), AppError> {
    consume_event(
        channel,
        "user-updated-media-service-queue".to_string(),
        move |event: UserUpdatedMessage| {
            let redis = redis.clone();
            let db = db.clone();
            Box::pin(async move { handle_user_updated(event, db, redis).await })
        },
    )
    .await
}

async fn handle_user_updated(
    event: UserUpdatedMessage,
    db: Database,
    redis: ::redis::Client,
) -> Result<(), AppError> {
    redis::hash_delete(&redis, "media", &event.id.to_string()).await?;
    User::update(event, &db).await?;

    Ok(())
}

async fn consume_event<T, F, Fut>(
    channel: &Channel,
    queue_name: String,
    handler: F,
) -> Result<(), AppError>
where
    T: for<'de> serde::Deserialize<'de> + Send + 'static,
    F: Fn(T) -> Fut + Send + 'static + Clone,
    Fut: Future<Output = Result<(), AppError>> + Send + 'static,
{
    let mut consumer = channel
        .basic_consume(
            &queue_name,
            &format!("{}-consumer", queue_name),
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    tokio::spawn(async move {
        while let Some(delivery_result) = consumer.next().await {
            match delivery_result {
                Ok(delivery) => {
                    let event: T = match serde_json::from_slice(&delivery.data) {
                        Ok(event) => event,
                        Err(err) => {
                            warn!("Failed to deserialize message from {}: {}", queue_name, err);
                            continue;
                        }
                    };

                    if let Err(err) = handler(event).await {
                        warn!("Failed to handle message from {}: {:?}", queue_name, err);
                    }

                    if let Err(err) = delivery.ack(BasicAckOptions::default()).await {
                        warn!(
                            "Failed to acknowledge message from {}: {:?}",
                            queue_name, err
                        );
                    }
                }
                Err(err) => {
                    warn!("Failed to consume message from {}: {:?}", queue_name, err);
                }
            }
        }
    });

    Ok(())
}
