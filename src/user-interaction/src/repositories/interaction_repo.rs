use crate::errors;
use crate::errors::redis_op_error;
use crate::services::cache_service::CacheService;
use async_trait::async_trait;
use redis::{pipe, AsyncCommands, ToRedisArgs};
use std::collections::HashMap;
use std::sync::Arc;
use ulid::Ulid;

#[async_trait]
pub trait InteractionRepository: Send + Sync {
    async fn get_user_interactions(
        &self,
        user_id: &str,
        fields: Vec<String>,
    ) -> Result<HashMap<String, bool>, errors::RedisError>;

    async fn is_user_liked(&self, user_id: &str, field: &str) -> Result<bool, errors::RedisError>;

    async fn like_post(&self, user_id: String, post_id: String)
        -> Result<bool, errors::RedisError>;

    async fn unlike_post(
        &self,
        user_id: String,
        post_id: String,
    ) -> Result<bool, errors::RedisError>;

    async fn add_view(&self, user_id: String, post_id: String) -> Result<bool, errors::RedisError>;

    async fn get_interactions(
        &self,
        key: &str,
        fields: &Vec<String>,
    ) -> Result<HashMap<Ulid, i32>, errors::RedisError>;

    async fn get_interaction(&self, key: &str, field: &str) -> Result<i32, errors::RedisError>;

    async fn delete_interactions(&self, post_id: &str) -> Result<(), errors::RedisError>;
    async fn delete_many_interactions(
        &self,
        posts_ids: &Vec<String>,
    ) -> Result<(), errors::RedisError>;
}

pub struct RedisInteractionRepository<C: CacheService + 'static> {
    redis_service: Arc<C>,
}

impl<C: CacheService + 'static> RedisInteractionRepository<C> {
    pub fn new(redis_service: Arc<C>) -> Self {
        Self { redis_service }
    }

    async fn _add_interaction(
        &self,
        conn: &mut redis::aio::MultiplexedConnection,
        user_id: &str,
        post_id: &str,
        interaction_type: &str,
        counter_key: &str,
    ) -> Result<bool, errors::RedisError> {
        let set_key = format!("user-interaction:posts:{}:{}", post_id, interaction_type);

        let is_member = self
            .redis_service
            .sismember_with_conn(conn, &set_key, user_id)
            .await?;
        if is_member {
            return Ok(false);
        }

        self.redis_service
            .sadd_with_conn(conn, &set_key, user_id)
            .await?;

        self.redis_service
            .hincr_with_conn(conn, counter_key, post_id, 1)
            .await?;

        Ok(true)
    }

    async fn _remove_interaction(
        &self,
        conn: &mut redis::aio::MultiplexedConnection,
        user_id: &str,
        post_id: &str,
        interaction_type: &str,
        counter_key: &str,
    ) -> Result<bool, errors::RedisError> {
        let set_key = format!("user-interaction:posts:{}:{}", post_id, interaction_type);

        let is_member = self
            .redis_service
            .sismember_with_conn(conn, &set_key, user_id)
            .await?;
        if !is_member {
            return Ok(false);
        }

        self.redis_service
            .srem_with_conn(conn, &set_key, user_id)
            .await?;

        self.redis_service
            .hincr_with_conn(conn, counter_key, post_id, -1)
            .await?;

        Ok(true)
    }
}

#[async_trait]
impl<C: CacheService + 'static> InteractionRepository for RedisInteractionRepository<C> {
    async fn get_user_interactions(
        &self,
        user_id: &str,
        fields: Vec<String>,
    ) -> Result<HashMap<String, bool>, errors::RedisError> {
        let mut conn = self.redis_service.open_redis_connection().await?;

        let mut pipe_cmd = pipe();
        for post_id in &fields {
            let like_set_key = format!("user-interaction:posts:{}:likes", post_id);
            pipe_cmd.cmd("SISMEMBER").arg(&like_set_key).arg(user_id);
        }

        let results: Vec<bool> = pipe_cmd
            .query_async(&mut conn)
            .await
            .map_err(|e| redis_op_error("PIPELINE_SISMEMBER", "multiple keys", e))?;

        let interaction_map = fields.into_iter().zip(results).collect();
        Ok(interaction_map)
    }

    async fn is_user_liked(
        &self,
        user_id: &str,
        post_id: &str,
    ) -> Result<bool, errors::RedisError> {
        let mut conn = self.redis_service.open_redis_connection().await?;
        let like_set_key = format!("user-interaction:posts:{}:likes", post_id);

        self.redis_service
            .sismember_with_conn(&mut conn, &like_set_key, user_id)
            .await
    }

    async fn like_post(
        &self,
        user_id: String,
        post_id: String,
    ) -> Result<bool, errors::RedisError> {
        let mut conn = self.redis_service.open_redis_connection().await?;
        self._add_interaction(
            &mut conn,
            &user_id,
            &post_id,
            "likes",
            "user-interaction:posts:likes",
        )
        .await
    }

    async fn unlike_post(
        &self,
        user_id: String,
        post_id: String,
    ) -> Result<bool, errors::RedisError> {
        let mut conn = self.redis_service.open_redis_connection().await?;

        let removed = self
            ._remove_interaction(
                &mut conn,
                &user_id,
                &post_id,
                "likes",
                "user-interaction:posts:likes",
            )
            .await?;

        if removed {
            self.redis_service
                .delete_all("user-interaction:replies", &format!("*{}*", post_id))
                .await?;
        }

        Ok(removed)
    }

    async fn add_view(&self, user_id: String, post_id: String) -> Result<bool, errors::RedisError> {
        let mut conn = self.redis_service.open_redis_connection().await?;
        self._add_interaction(
            &mut conn,
            &user_id,
            &post_id,
            "views",
            "user-interaction:posts:views",
        )
        .await
    }

    async fn get_interactions(
        &self,
        key: &str,
        fields: &Vec<String>,
    ) -> Result<HashMap<Ulid, i32>, errors::RedisError> {
        let mut conn = self.redis_service.open_redis_connection().await?;

        let value = serde_json::to_string(fields)?;
        let results: Vec<Option<i32>> = conn
            .hget(key, value)
            .await
            .map_err(|e| redis_op_error("HMGET", key, e))?;

        let interaction_map: HashMap<Ulid, i32> = fields
            .iter()
            .zip(results.into_iter())
            .filter_map(|(field, value)| {
                Ulid::from_string(field)
                    .ok()
                    .map(|ulid| (ulid, value.unwrap_or_default()))
            })
            .collect();

        Ok(interaction_map)
    }

    async fn get_interaction(&self, key: &str, field: &str) -> Result<i32, errors::RedisError> {
        let mut conn = self.redis_service.open_redis_connection().await?;
        let val: i32 = self
            .redis_service
            .hget_with_conn(&mut conn, key, field)
            .await?
            .unwrap_or(0);
        Ok(val)
    }

    async fn delete_interactions(&self, post_id: &str) -> Result<(), errors::RedisError> {
        let mut conn = self.redis_service.open_redis_connection().await?;

        for key in &[
            "user-interaction:posts:likes",
            "user-interaction:posts:views",
        ] {
            self.redis_service
                .hdel_with_conn(&mut conn, key, post_id)
                .await?;
        }

        self.redis_service
            .delete_all_with_conn(&mut conn,"user-interaction:replies", &format!("*{}*", post_id))
            .await?;

        for suffix in &["likes", "views"] {
            let set_key = format!("user-interaction:posts:{}:{}", post_id, suffix);
            let members = self
                .redis_service
                .smembers_with_conn(&mut conn, &set_key)
                .await?;
            if !members.is_empty() {
                for member in members {
                    self.redis_service
                        .srem_with_conn(&mut conn, &set_key, &member)
                        .await?;
                }
            }
        }

        Ok(())
    }

    async fn delete_many_interactions(
        &self,
        post_ids: &Vec<String>,
    ) -> Result<(), errors::RedisError> {
        if post_ids.is_empty() {
            return Ok(());
        }
        let mut conn = self.redis_service.open_redis_connection().await?;

        let likes_key = "user-interaction:posts:likes";
        let views_key = "user-interaction:posts:views";

        conn.hdel(likes_key, post_ids)
            .await
            .map_err(|e| redis_op_error("HDEL", likes_key, e))?;
        conn.hdel(views_key, post_ids)
            .await
            .map_err(|e| redis_op_error("HDEL", views_key, e))?;

        for post_id in post_ids {
            self.redis_service
                .delete_all_with_conn(
                    &mut conn,
                    "user-interaction:replies",
                    &format!("*{}*", post_id),
                )
                .await?;
        }

        let mut pipe_smembers = pipe();
        for post_id in post_ids {
            pipe_smembers
                .cmd("SMEMBERS")
                .arg(format!("user-interaction:posts:{}:likes", post_id));
            pipe_smembers
                .cmd("SMEMBERS")
                .arg(format!("user-interaction:posts:{}:views", post_id));
        }

        let smembers_results: Vec<Vec<String>> = pipe_smembers
            .query_async(&mut conn)
            .await
            .map_err(|e| redis_op_error("PIPELINE_SMEMBERS", "like/view set keys", e))?;

        let mut idx = 0;
        let mut pipe_srem = pipe();
        for post_id in post_ids {
            let like_key = format!("user-interaction:posts:{}:likes", post_id);
            let view_key = format!("user-interaction:posts:{}:views", post_id);

            let like_members = smembers_results.get(idx).cloned().unwrap_or_default();
            let view_members = smembers_results.get(idx + 1).cloned().unwrap_or_default();
            idx += 2;

            if !like_members.is_empty() {
                pipe_srem.cmd("SREM").arg(&like_key).arg(like_members);
            }
            if !view_members.is_empty() {
                pipe_srem.cmd("SREM").arg(&view_key).arg(view_members);
            }
        }

        pipe_srem
            .query_async(&mut conn)
            .await
            .map_err(|e| redis_op_error("PIPELINE_SREM", "like/view membership sets", e))?;

        Ok(())
    }
}
