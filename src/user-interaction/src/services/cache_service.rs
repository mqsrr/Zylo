use crate::{errors, settings};
use async_trait::async_trait;
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, AsyncIter, Client, ExpireOption};
use serde::de::DeserializeOwned;
use serde::Serialize;

#[async_trait]
pub trait CacheService: Send + Sync {
    async fn open_redis_connection(&self) -> Result<MultiplexedConnection, errors::RedisError>;

    async fn hfind<T: DeserializeOwned>(
        &self,
        key: &str,
        pattern: &str,
    ) -> Result<Option<T>, errors::RedisError>;

    async fn hget<T: DeserializeOwned>(
        &self,
        key: &str,
        field: &str,
    ) -> Result<Option<T>, errors::RedisError>;

    async fn hset<T: Serialize + Sync + Send>(
        &self,
        key: &str,
        field: &str,
        value: &T,
    ) -> Result<(), errors::RedisError>;

    async fn sadd(&self, key: &str, member: &str) -> Result<(), errors::RedisError>;

    async fn sismember(&self, key: &str, member: &str) -> Result<bool, errors::RedisError>;

    async fn srem(&self, key: &str, member: &str) -> Result<(), errors::RedisError>;

    async fn delete(&self, key: &str, fields: &str) -> Result<(), errors::RedisError>;

    async fn delete_all(&self, key: &str, pattern: &str) -> Result<(), errors::RedisError>;

    async fn sismember_with_conn(
        &self,
        conn: &mut MultiplexedConnection,
        key: &str,
        member: &str,
    ) -> Result<bool, errors::RedisError>;

    async fn sadd_with_conn(
        &self,
        conn: &mut MultiplexedConnection,
        key: &str,
        member: &str,
    ) -> Result<(), errors::RedisError>;

    async fn srem_with_conn(
        &self,
        conn: &mut MultiplexedConnection,
        key: &str,
        member: &str,
    ) -> Result<(), errors::RedisError>;

    async fn hincr_with_conn(
        &self,
        conn: &mut MultiplexedConnection,
        key: &str,
        field: &str,
        increment: i32,
    ) -> Result<(), errors::RedisError>;

    async fn hget_with_conn<T: DeserializeOwned>(
        &self,
        conn: &mut MultiplexedConnection,
        key: &str,
        field: &str,
    ) -> Result<Option<T>, errors::RedisError>;

    async fn hdel_with_conn(
        &self,
        conn: &mut MultiplexedConnection,
        key: &str,
        field: &str,
    ) -> Result<(), errors::RedisError>;

    async fn smembers_with_conn(
        &self,
        conn: &mut MultiplexedConnection,
        key: &str,
    ) -> Result<Vec<String>, errors::RedisError>;

    async fn delete_all_with_conn(
        &self,
        conn: &mut MultiplexedConnection,
        key: &str,
        pattern: &str,
    ) -> Result<(), errors::RedisError>;
}

pub struct RedisCacheService {
    redis: Client,
    config: settings::Redis,
}

impl RedisCacheService {
    pub fn new(config: settings::Redis) -> Result<Self, errors::RedisError> {
        let redis = Client::open(config.uri.to_string())
            .map_err(|e| errors::redis_op_error("CONNECTION", "N/A", e))?;

        Ok(Self { redis, config })
    }
}

#[async_trait]
impl CacheService for RedisCacheService {
    async fn open_redis_connection(&self) -> Result<MultiplexedConnection, errors::RedisError> {
        self.redis
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| errors::redis_op_error("CONNECT", "N/A", e))
    }

    async fn hfind<T: DeserializeOwned>(
        &self,
        key: &str,
        pattern: &str,
    ) -> Result<Option<T>, errors::RedisError> {
        let mut conn = self.open_redis_connection().await?;

        let mut iter: AsyncIter<String> = conn
            .hscan_match(key, pattern)
            .await
            .map_err(|e| errors::redis_op_error("SCAN", key, e))?;

        while let Some(value) = iter.next_item().await {
            if let Ok(result) = serde_json::from_str(&value) {
                return Ok(Some(result));
            }
        }

        Ok(None)
    }

    async fn hget<T: DeserializeOwned>(
        &self,
        key: &str,
        field: &str,
    ) -> Result<Option<T>, errors::RedisError> {
        let mut conn = self.open_redis_connection().await?;
        self.hget_with_conn(&mut conn, key, field).await
    }

    async fn hset<T: Serialize + Sync + Send>(
        &self,
        key: &str,
        field: &str,
        value: &T,
    ) -> Result<(), errors::RedisError> {
        let mut conn = self.open_redis_connection().await?;

        conn.hset(key, field, serde_json::to_string(value)?)
            .await
            .map_err(|e| errors::redis_op_error("HSET", key, e))?;

        conn.hexpire(key, self.config.expire_time as i64, ExpireOption::GT, field)
            .await
            .map_err(|e| errors::redis_op_error("HEXPIRE", key, e))?;

        Ok(())
    }

    async fn sadd(&self, key: &str, member: &str) -> Result<(), errors::RedisError> {
        let mut conn = self.open_redis_connection().await?;
        self.sadd_with_conn(&mut conn, key, member).await
    }

    async fn sismember(&self, key: &str, member: &str) -> Result<bool, errors::RedisError> {
        let mut conn = self.open_redis_connection().await?;
        self.sismember_with_conn(&mut conn, key, member).await
    }

    async fn srem(&self, key: &str, member: &str) -> Result<(), errors::RedisError> {
        let mut conn = self.open_redis_connection().await?;
        self.srem_with_conn(&mut conn, key, member).await
    }

    async fn delete(&self, key: &str, field: &str) -> Result<(), errors::RedisError> {
        let mut conn = self.open_redis_connection().await?;
        self.hdel_with_conn(&mut conn, key, field).await
    }

    async fn delete_all(&self, key: &str, pattern: &str) -> Result<(), errors::RedisError> {
        let mut conn = self.open_redis_connection().await?;
        self.delete_all_with_conn(&mut conn, key, pattern).await
    }

    async fn sismember_with_conn(
        &self,
        conn: &mut MultiplexedConnection,
        key: &str,
        member: &str,
    ) -> Result<bool, errors::RedisError> {
        conn.sismember(key, member)
            .await
            .map_err(|e| errors::redis_op_error("SISMEMBER", key, e))
    }

    async fn sadd_with_conn(
        &self,
        conn: &mut MultiplexedConnection,
        key: &str,
        member: &str,
    ) -> Result<(), errors::RedisError> {
        conn.sadd(key, member)
            .await
            .map_err(|e| errors::redis_op_error("SADD", key, e))?;

        Ok(())
    }

    async fn srem_with_conn(
        &self,
        conn: &mut MultiplexedConnection,
        key: &str,
        member: &str,
    ) -> Result<(), errors::RedisError> {
        conn.srem(key, member)
            .await
            .map_err(|e| errors::redis_op_error("SREM", key, e))?;
        Ok(())
    }

    async fn hincr_with_conn(
        &self,
        conn: &mut MultiplexedConnection,
        key: &str,
        field: &str,
        increment: i32,
    ) -> Result<(), errors::RedisError> {
        conn.hincr(key, field, increment)
            .await
            .map_err(|e| errors::redis_op_error("HINCR", key, e))?;
        Ok(())
    }

    async fn hget_with_conn<T: DeserializeOwned>(
        &self,
        conn: &mut MultiplexedConnection,
        key: &str,
        field: &str,
    ) -> Result<Option<T>, errors::RedisError> {
        let cached: Option<String> = conn.hget(key, field)
            .await
            .map_err(|e| errors::redis_op_error("HGET", key, e))?;
        
        if let Some(cached) = cached
        {
            return serde_json::from_str(&cached).map_or(Ok(None), |v| Ok(Some(v)));
        }

        Ok(None)
    }

    async fn hdel_with_conn(
        &self,
        conn: &mut MultiplexedConnection,
        key: &str,
        field: &str,
    ) -> Result<(), errors::RedisError> {
        conn.hdel(key, field)
            .await
            .map_err(|e| errors::redis_op_error("HDEL", key, e))?;
        Ok(())
    }

    async fn smembers_with_conn(
        &self,
        conn: &mut MultiplexedConnection,
        key: &str,
    ) -> Result<Vec<String>, errors::RedisError> {
        conn.smembers(key)
            .await
            .map_err(|e| errors::redis_op_error("SMEMBERS", key, e))
    }

    async fn delete_all_with_conn(
        &self,
        conn: &mut MultiplexedConnection,
        key: &str,
        pattern: &str,
    ) -> Result<(), errors::RedisError> {
        let mut del_conn = conn.clone();
        let mut fields_to_delete: Vec<String> = Vec::new();

        let mut iter: AsyncIter<String> = conn
            .hscan_match(key, pattern)
            .await
            .map_err(|e| errors::redis_op_error("HSCAN", key, e))?;

        while let Some(field) = iter.next_item().await {
            fields_to_delete.push(field);
        }

        del_conn
            .hdel(key, fields_to_delete)
            .await
            .map_err(|r| errors::redis_op_error("HDEL", key, r))?;

        Ok(())
    }
}
