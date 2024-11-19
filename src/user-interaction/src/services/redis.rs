use crate::errors::AppError;
use crate::setting::Redis;
use redis::{pipe, AsyncCommands, AsyncIter, Client, ConnectionLike, ExpireOption};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
use ulid::Ulid;
use crate::services::backup_worker;

pub async fn init_client(config: &Redis) -> Client{
    let mut client = Client::open(config.uri.to_string()).expect("Failed to created redis client");
    let is_connected = client.check_connection();
    if !is_connected { 
        panic!("Failed to connect to Redis");
    }

    backup_worker::start_worker(config).await.unwrap_or_else(|e| panic!("{:?}", e));
    client
}

pub async fn hash_get<T: DeserializeOwned + Sync + Send>(redis: &Client, key: &str, field: &str) -> Result<Option<T>, AppError> {
    let mut conn = redis.get_multiplexed_async_connection().await?;
    let cached_value: Option<String> = conn.hget(key, field).await?;
    
    Ok(serde_json::from_str::<T>(&cached_value.unwrap_or_default()).ok())
}

pub async fn hash_delete(redis: &Client, key: &str, field: &str) -> Result<(), AppError> {
    let mut conn = redis.get_multiplexed_async_connection().await?;
    conn.hdel(key, field).await?;
    
    Ok(())
}

pub async fn hash_set<T: Serialize + Sync + Send>(redis: &Client, key: &str, field: &str, value: &T, expire: i64) -> Result<(), AppError> {
    let mut conn = redis.get_multiplexed_async_connection().await?;
    
    conn.hset(key, field, serde_json::to_string(value).unwrap()).await?;
    conn.hexpire(key, expire, ExpireOption::NONE, field).await?;
    Ok(())
}

pub async fn hash_delete_all(redis: &Client, key: &str, pattern: &str) -> Result<(), AppError> {
    let mut conn = redis.get_multiplexed_async_connection().await?;
    let mut pipe_conn = conn.clone();
    
    let mut pipe = pipe();
    let mut fields: AsyncIter<String> = conn.hscan_match(key, pattern).await?;
    
    while let Some(field) = fields.next_item().await {
        pipe.hdel(key, field);
    }
    
    pipe.query_async(&mut pipe_conn).await?;
    Ok(())
}

pub async fn hash_scan<T: DeserializeOwned + Sync + Send>(redis: &Client, key: &str, pattern: &str) -> Result<Option<T>, AppError> {
    let mut conn = redis.get_multiplexed_async_connection().await?;
    let mut fields: AsyncIter<String> = conn.hscan_match(key, pattern).await?;
    
    if let Some(field) = fields.next_item().await {
        let value = hash_get::<T>(&redis, key, &field).await?;
        return Ok(value);
    }
    
    Ok(None)
}

pub async fn is_user_interacted(redis: &Client, user_id: &str, fields: Vec<String>) -> Result<HashMap<String, bool>, AppError> {
    let mut conn = redis.get_multiplexed_async_connection().await?;
    let mut pipe = pipe();

    for field in &fields {
        pipe.cmd("SISMEMBER")
            .arg(format!("user-interaction:posts:{}:likes", field))
            .arg(user_id);
    }

    let results: Vec<bool> = pipe.query_async(&mut conn).await?;
    let hash: HashMap<String, bool> = fields.into_iter().zip(results.into_iter()).collect();

    Ok(hash)
}

pub async fn like_post(redis: &Client, user_id: String, post_id: String) -> Result<(), AppError> {
    let mut conn = redis.get_multiplexed_async_connection().await?;
    let is_member: bool = conn.sismember(format!("user-interaction:posts:{}:likes", &post_id), &user_id).await?;
    if is_member {
        return Err(AppError::UserInteractionAlreadyAssigned);
    }
    
    conn.sadd(format!("user-interaction:posts:{}:likes", &post_id), &user_id).await?;
    conn.hincr("user-interaction:posts:likes", &post_id, 1).await?;

    hash_delete_all(&redis, "user-interaction:replies", &format!("*{}*", &post_id)).await?;
    Ok(())
}

pub async fn unlike_post(redis: &Client, user_id: String, post_id: String) -> Result<(), AppError> {
    let mut conn = redis.get_multiplexed_async_connection().await?;
    let is_member: bool = conn.sismember(format!("user-interaction:posts:{}:likes", &post_id), &user_id).await?;
    if !is_member {
        return Err(AppError::UserInteractionNotFound);
    }
    
    conn.srem(format!("user-interaction:posts:{}:likes", &post_id), &user_id).await?;
    conn.hincr("user-interaction:posts:likes", &post_id, -1).await?;

    hash_delete_all(redis,"user-interaction:replies", &format!("*{}*", post_id)).await?;
    Ok(())
}

pub async fn add_view(redis: &Client, user_id: String, post_id: String) -> Result<(), AppError> {
    let mut conn = redis.get_multiplexed_async_connection().await?;
    let is_member: bool = conn.sismember(format!("user-interaction:posts:{}:views", &post_id), &user_id).await?;
    if is_member {
        return Err(AppError::UserInteractionAlreadyAssigned);
    }

    conn.sadd(format!("user-interaction:posts:{}:views", &post_id), &user_id).await?;
    conn.hincr("user-interaction:posts:views", &post_id, 1).await?;

    hash_delete_all(redis,"user-interaction:replies", &format!("*{}*", post_id)).await?;
    Ok(())
}

pub async fn get_interactions(redis: &Client, key: &str, fields: &Vec<String>) -> Result<HashMap<Ulid, i32>, AppError> {
    let mut conn = redis.get_multiplexed_async_connection().await?;
    let cached_values: Vec<Option<String>>= conn.hget(key, fields).await.unwrap_or_default();
    
    let mut interaction_map: HashMap<Ulid, i32> = HashMap::new();
    
    for (i, value) in cached_values.into_iter().enumerate() {
        interaction_map.insert(Ulid::from_string(fields[i].as_str()).unwrap(), value.unwrap_or(String::from("0")).parse::<i32>().unwrap());
    }
    
    Ok(interaction_map)
}

pub async fn delete_interactions(redis: &Client, post_id: &str) -> Result<(), AppError> {
    let mut conn = redis.get_multiplexed_async_connection().await?;
    conn.hdel("user-interaction:posts:likes", post_id).await?;
    conn.hdel("user-interaction:posts:views", post_id).await?;
    
    hash_delete_all(redis,"user-interaction:replies", &format!("*{}*", post_id)).await?;
    
    let likes_key = format!("user-interaction:posts:{}:likes", post_id);
    let likes_members: Vec<String> = conn.smembers(&likes_key).await?;
    if !likes_members.is_empty() {
        conn.srem(&likes_key, likes_members).await?;
    }

    let views_key = format!("user-interaction:posts:{}:views", post_id);
    let views_members: Vec<String> = conn.smembers(&views_key).await?;
    if !views_members.is_empty() {
        conn.srem(&views_key, views_members).await?;
    }

    Ok(())
}
