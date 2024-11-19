use futures_util::stream::StreamExt;
use std::future::Future;
use lapin::{BasicProperties, Channel, Connection, ConnectionProperties};
use lapin::options::{BasicAckOptions, BasicConsumeOptions, BasicPublishOptions, ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions};
use lapin::types::FieldTable;
use log::{error, warn};
use serde::Serialize;
use sqlx::{PgPool};
use ulid::Ulid;
use crate::errors::AppError;
use crate::models::amq_message::{PostDeletedMessage, UserCreatedMessage, UserDeletedMessage, UserUpdatedMessage};
use crate::setting::{RabbitMq};
use crate::services::redis;
use crate::utils::constants::{POST_EXCHANGE_NAME, USER_EXCHANGE_NAME};


pub async fn open_amq_connection(config: &RabbitMq) -> Channel {
    let connection = Connection::connect(&config.uri, ConnectionProperties::default()).await.unwrap();
    let channel = connection.create_channel().await.unwrap();

    declare_exchanges(&channel).await.unwrap();
    declare_queues(&channel).await.unwrap();

    channel
}

async fn declare_exchanges(channel: &Channel) -> Result<(), AppError> {
    let exchange_options = ExchangeDeclareOptions { durable: true, ..Default::default() };

    channel.exchange_declare(POST_EXCHANGE_NAME, lapin::ExchangeKind::Direct, exchange_options, FieldTable::default()).await?;
    channel.exchange_declare(USER_EXCHANGE_NAME, lapin::ExchangeKind::Direct, exchange_options, FieldTable::default()).await?;

    Ok(())
}

async fn declare_queues(channel: &Channel) -> Result<(), AppError> {
    let queue_options = QueueDeclareOptions { durable: true, ..Default::default() };
    let queue_map = vec![
        ("post-deleted-user-interaction-queue", POST_EXCHANGE_NAME, "post.deleted"),
        ("user-created-user-interaction-queue", USER_EXCHANGE_NAME, "user.created"),
        ("user-updated-user-interaction-queue", USER_EXCHANGE_NAME, "user.updated"),
        ("user-deleted-user-interaction-queue", USER_EXCHANGE_NAME, "user.deleted"),
    ];

    for (queue_name, exchange_name, routing_key) in queue_map {
        channel.queue_declare(queue_name, queue_options, FieldTable::default()).await?;
        channel.queue_bind(queue_name, exchange_name, routing_key, QueueBindOptions::default(), FieldTable::default()).await?;
    }

    Ok(())
}

pub async fn publish_event<T: Serialize>(channel: &Channel, exchange_name: &str, routing_key: &str, event: &T) -> Result<(), AppError> {
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

async fn fetch_top_level_reply_to_ids(db: &PgPool, user_id: &Ulid) -> Vec<String> {
    sqlx::query!(
            r#"
            SELECT DISTINCT reply_to_id 
            FROM replies
            WHERE user_id = $1 
              AND reply_to_id NOT IN (SELECT id FROM replies)
            "#, 
            &user_id.to_bytes()
        )
        .fetch_all(db)
        .await
        .map_err(|e| error!("{:?}", e)).unwrap()
        .into_iter()
        .map(|row| Ulid::from_bytes(row.reply_to_id.try_into().unwrap()).to_string())
        .collect::<Vec<_>>()
}

pub async fn consume_post_deleted(channel: &Channel, db: PgPool, redis: ::redis::Client) -> Result<(), AppError> {
    consume_event(
        channel,
        "post-deleted-user-interaction-queue".to_string(),
        move |event: PostDeletedMessage| {
            let db = db.clone();
            let redis = redis.clone();
            Box::pin(async move {
                handle_post_deleted(event, db, redis).await
            })
        },
    ).await
}


pub async fn handle_post_deleted(event: PostDeletedMessage, db: PgPool, redis: ::redis::Client) -> Result<(), AppError> {
    sqlx::query!(
            r#"
            WITH RECURSIVE reply_tree AS (
                SELECT id, user_id, reply_to_id, content, created_at
                FROM replies
                WHERE reply_to_id = $1
            
                UNION ALL
            
                SELECT r.id, r.user_id, r.reply_to_id, r.content, r.created_at
                FROM replies r
                INNER JOIN reply_tree rt ON rt.id = r.reply_to_id
            )
            DELETE FROM replies WHERE id IN (SELECT id FROM reply_tree);
            "#, &event.post_id.to_bytes())
        .execute(&db).await.map_err(|e| error!("{:?}", e)).unwrap();
    
    redis::delete_interactions(&redis, &event.post_id.to_string()).await?;
    Ok(())
}

pub async fn consume_user_created(channel: &Channel, db: PgPool) -> Result<(), AppError> {
    consume_event(
        channel,
        "user-created-user-interaction-queue".to_string(),
        move |event: UserCreatedMessage| {
            let db = db.clone();
            Box::pin(async move {
                handle_user_created(event, db).await
            })
        },
    ).await
}

pub async fn handle_user_created(event: UserCreatedMessage, db: PgPool) -> Result<(), AppError> {
    sqlx::query!(
            r#"
            INSERT INTO users (id, username, name)
            VALUES ($1, $2, $3);
            "#, &event.id.to_bytes(), event.username, event.name)
        .execute(&db).await.map_err(|e| error!("{:?}", e)).unwrap();

    Ok(())
}

pub async fn consume_user_updated(channel: &Channel, db: PgPool, redis: ::redis::Client) -> Result<(), AppError> {
    consume_event(
        channel,
        "user-updated-user-interaction-queue".to_string(),
        move |event: UserUpdatedMessage| {
            let db = db.clone();
            let redis = redis.clone();
            Box::pin(async move {
                handle_user_updated(event, db, redis).await
            })
        },
    ).await
}

pub async fn handle_user_updated(event: UserUpdatedMessage, db: PgPool , redis: ::redis::Client) -> Result<(), AppError> {
    let posts_ids = fetch_top_level_reply_to_ids(&db, &event.id).await;
    
    sqlx::query!(
            r#"
           UPDATE users
           SET name = $2, bio = $3, location = $4
           WHERE id = $1;
            "#, &event.id.to_bytes(), event.name, event.bio, event.location)
        .execute(&db).await.map_err(|e| error!("{:?}", e)).unwrap();

    for post_id in &posts_ids {
        redis::hash_delete(&redis, "user-interaction:replies", post_id).await?;
    }
    Ok(())
}

pub async fn consume_user_deleted(channel: &Channel, db: PgPool, redis: ::redis::Client) -> Result<(), AppError> {
    consume_event(
        channel,
        "user-deleted-user-interaction-queue".to_string(),
        move |event: UserDeletedMessage| {
            let db = db.clone();
            let redis = redis.clone();
            Box::pin(async move {
                handle_user_deleted(event, db, redis).await
            })
        },
    ).await
}

pub async fn handle_user_deleted(event: UserDeletedMessage, db: PgPool, redis: ::redis::Client) -> Result<(), AppError> {
    let posts_ids = fetch_top_level_reply_to_ids(&db, &event.id).await;
    
    sqlx::query!(
            r#"
            DELETE FROM users
            WHERE id = $1;
            "#, &event.id.to_bytes())
        .execute(&db)
        .await
        .map_err(|e| error!("{:?}", e)).unwrap();
    for post_id in &posts_ids {
        redis::hash_delete(&redis, "user-interaction:replies", post_id).await?;
    }
    
    Ok(())
}

async fn consume_event<T, F, Fut>(channel: &Channel, queue_name: String, handler: F) -> Result<(), AppError>
where
    T: for<'de> serde::Deserialize<'de> + Send + 'static,
    F: Fn(T) -> Fut + Send + 'static + Clone,
    Fut: Future<Output=Result<(), AppError>> + Send + 'static,
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
                        warn!("Failed to acknowledge message from {}: {:?}", queue_name, err);
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
