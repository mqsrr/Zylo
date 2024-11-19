use crate::errors::AppError;
use crate::settings::Redis;
use redis::{AsyncCommands, Client, ConnectionLike};
use serde::de::DeserializeOwned;
use serde::Serialize;

pub async fn create_client(config: &Redis) -> Client {
    let mut client = Client::open(config.uri.to_string()).expect("Failed to create redis client");

    let is_connected = client.check_connection();
    if !is_connected {
        panic!("Failed to connect to Redis");
    };

    client
}

pub async fn hash_set<T: Serialize + Send + Sync>(
    redis: &Client,
    key: &str,
    field: &str,
    value: &T,
    expire: i64,
) -> Result<(), AppError> {
    let mut conn = redis.get_multiplexed_async_connection().await?;
    conn.hset(key, field, serde_json::to_string(value).unwrap()).await?;

    conn.hexpire(key, expire, redis::ExpireOption::NX, field).await?;
    Ok(())
}

pub async fn hash_delete(redis: &Client, key: &str, field: &str) -> Result<(), AppError> {
    let mut conn = redis.get_multiplexed_async_connection().await?;
    conn.hdel(key, field).await?;

    Ok(())
}

pub async fn hash_get<T: DeserializeOwned>(
    redis: &Client,
    key: &str,
    field: &str,
) -> Result<Option<T>, AppError> {
    let mut conn = redis.get_multiplexed_async_connection().await?;
    let cached_value: String = conn.hget(key, field).await.unwrap_or_default();

    Ok(serde_json::from_str::<T>(&cached_value).ok())
}
