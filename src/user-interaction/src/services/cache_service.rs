use crate::{errors, settings};
use async_trait::async_trait;
use redis::aio::MultiplexedConnection;
use redis::{cmd, pipe, AsyncCommands, AsyncIter, Client, ExpireOption};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;

#[async_trait]
pub trait CacheService: Send + Sync {
    async fn get_conn(&self) -> Result<MultiplexedConnection, errors::RedisError>;

    async fn hfind<T: DeserializeOwned>(
        &self,
        key: &str,
        pattern: &str,
    ) -> Result<Option<T>, errors::RedisError>;

    async fn hfind_keys(&self, key: &str, pattern: &str)
        -> Result<Vec<String>, errors::RedisError>;

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

    async fn hdel(&self, key: &str, fields: &[String]) -> Result<(), errors::RedisError>;

    async fn pfadd(&self, key: &str, element: &str) -> Result<bool, errors::RedisError>;
    async fn pfcount(&self, key: &str) -> Result<u64, errors::RedisError>;
    async fn pfcount_many(
        &self,
        keys: &[String],
    ) -> Result<HashMap<String, u64>, errors::RedisError>;

    async fn sadd(&self, key: &str, member: &str) -> Result<bool, errors::RedisError>;
    async fn srem(&self, key: &str, member: &str) -> Result<bool, errors::RedisError>;
    async fn scard(&self, key: &str) -> Result<u64, errors::RedisError>;
    async fn scard_many(&self, keys: &[String])
        -> Result<HashMap<String, u64>, errors::RedisError>;
    async fn sismember(&self, key: &str, member: &str) -> Result<bool, errors::RedisError>;
    async fn sismember_many(
        &self,
        keys: &[String],
        member: &str,
    ) -> Result<HashMap<String, bool>, errors::RedisError>;

    async fn del(&self, keys: &[String]) -> Result<(), errors::RedisError>;
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
    async fn get_conn(&self) -> Result<MultiplexedConnection, errors::RedisError> {
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
        let mut conn = self.get_conn().await?;
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

    async fn hfind_keys(
        &self,
        key: &str,
        pattern: &str,
    ) -> Result<Vec<String>, errors::RedisError> {
        let mut conn = self.get_conn().await?;
        let keys: Vec<String> = cmd("HSCAN")
            .arg(key)
            .arg(pattern)
            .arg("NOVALUES")
            .query_async(&mut conn)
            .await
            .map_err(|e| errors::redis_op_error("HSCAN", key, e))?;

        Ok(keys)
    }

    async fn hget<T: DeserializeOwned>(
        &self,
        key: &str,
        field: &str,
    ) -> Result<Option<T>, errors::RedisError> {
        let mut conn = self.get_conn().await?;

        let cached_result: Option<String> = conn
            .hget(key, field)
            .await
            .map_err(|e| errors::redis_op_error("HGET", key, e))?;

        let cached_result = match cached_result {
            None => return Ok(None),
            Some(cache) => cache,
        };

        Ok(serde_json::from_str(&cached_result)?)
    }

    async fn hset<T: Serialize + Sync + Send>(
        &self,
        key: &str,
        field: &str,
        value: &T,
    ) -> Result<(), errors::RedisError> {
        let mut conn = self.get_conn().await?;

        conn.hset::<_, _, _, ()>(key, field, serde_json::to_string(value)?)
            .await
            .map_err(|e| errors::redis_op_error("HSET", key, e))?;

        conn.hexpire::<_, _, ()>(key, self.config.expire_time as i64, ExpireOption::GT, field)
            .await
            .map_err(|e| errors::redis_op_error("HEXPIRE", key, e))?;

        Ok(())
    }

    async fn hdel(&self, key: &str, fields: &[String]) -> Result<(), errors::RedisError> {
        let mut conn = self.get_conn().await?;

        conn.hdel::<_, _, ()>(key, fields)
            .await
            .map_err(|e| errors::redis_op_error("HDEL", key, e))
    }

    async fn pfadd(&self, key: &str, element: &str) -> Result<bool, errors::RedisError> {
        let mut conn = self.get_conn().await?;

        conn.pfadd(key, element)
            .await
            .map_err(|e| errors::redis_op_error("PFADD", key, e))
    }

    async fn pfcount(&self, key: &str) -> Result<u64, errors::RedisError> {
        let mut conn = self.get_conn().await?;
        conn.pfcount(key)
            .await
            .map_err(|e| errors::redis_op_error("PIPE PFCOUNT", "multiple", e))
    }

    async fn pfcount_many(
        &self,
        keys: &[String],
    ) -> Result<HashMap<String, u64>, errors::RedisError> {
        let mut conn = self.get_conn().await?;
        let mut pipe = pipe();

        for key in keys {
            pipe.pfcount(key);
        }

        let views: Vec<u64> = pipe
            .atomic()
            .query_async(&mut conn)
            .await
            .map_err(|e| errors::redis_op_error("PIPE PFCOUNT", "multiple", e))?;

        let mut map = HashMap::new();
        for (idx, key) in keys.iter().enumerate() {
            map.insert(key.to_string(), views[idx]);
        }

        Ok(map)
    }

    async fn sadd(&self, key: &str, member: &str) -> Result<bool, errors::RedisError> {
        let mut conn = self.get_conn().await?;
        conn.sadd(key, member)
            .await
            .map_err(|e| errors::redis_op_error("SADD", key, e))
    }

    async fn srem(&self, key: &str, member: &str) -> Result<bool, errors::RedisError> {
        let mut conn = self.get_conn().await?;
        conn.srem(key, member)
            .await
            .map_err(|e| errors::redis_op_error("SREM", key, e))
    }

    async fn scard(&self, key: &str) -> Result<u64, errors::RedisError> {
        let mut conn = self.get_conn().await?;

        conn.scard(key)
            .await
            .map_err(|e| errors::redis_op_error("SCARD", key, e))
    }

    async fn scard_many(
        &self,
        keys: &[String],
    ) -> Result<HashMap<String, u64>, errors::RedisError> {
        let mut conn = self.get_conn().await?;
        let mut pipe = pipe();

        pipe.atomic();
        for key in keys {
            pipe.scard(key);
        }

        let likes: Vec<u64> = pipe
            .query_async(&mut conn)
            .await
            .map_err(|e| errors::redis_op_error("PIPE SCARD", "multiple", e))?;

        let mut map = HashMap::new();
        for (idx, key) in keys.iter().enumerate() {
            map.insert(key.to_string(), likes[idx]);
        }

        Ok(map)
    }

    async fn sismember(&self, key: &str, member: &str) -> Result<bool, errors::RedisError> {
        let mut conn = self.get_conn().await?;
        conn.sismember(key, member)
            .await
            .map_err(|e| errors::redis_op_error("SISMEMBER", key, e))
    }

    async fn sismember_many(
        &self,
        keys: &[String],
        member: &str,
    ) -> Result<HashMap<String, bool>, errors::RedisError> {
        let mut conn = self.get_conn().await?;
        let mut pipe = pipe();

        for key in keys {
            pipe.sismember(key, member);
        }

        let results: Vec<bool> = pipe
            .atomic()
            .query_async(&mut conn)
            .await
            .map_err(|e| errors::redis_op_error("PIPE SISMEMBER", "multiple", e))?;

        let mut map = HashMap::new();
        for (idx, key) in keys.iter().enumerate() {
            map.insert(key.to_string(), results[idx]);
        }

        Ok(map)
    }

    async fn del(&self, keys: &[String]) -> Result<(), errors::RedisError> {
        let mut conn = self.get_conn().await?;
        conn.del(keys)
            .await
            .map_err(|e| errors::redis_op_error("DEL", "multiple", e))
    }
}
