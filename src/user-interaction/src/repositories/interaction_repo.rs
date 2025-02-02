use crate::errors;
use crate::services::cache_service::CacheService;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use ulid::Ulid;

#[async_trait]
pub trait InteractionRepository: Send + Sync {
    async fn is_liked(
        &self,
        resource_key: &str,
        user_id: &Ulid,
    ) -> Result<bool, errors::RedisError>;
    async fn is_many_liked(
        &self,
        resource_keys: &[String],
        user_id: &Ulid,
    ) -> Result<HashMap<String, bool>, errors::RedisError>;
    async fn get_likes(&self, resource_key: &str) -> Result<u64, errors::RedisError>;
    async fn get_many_likes(&self, resource_key: &[String]) -> Result<HashMap<String, u64>, errors::RedisError>;
    async fn like(&self, resource_key: &str, user_id: &Ulid) -> Result<bool, errors::RedisError>;
    async fn unlike(&self, resource_key: &str, user_id: &Ulid) -> Result<bool, errors::RedisError>;

    async fn view(&self, resource_key: &str, user_id: &Ulid) -> Result<bool, errors::RedisError>;
    async fn get_views(&self, resource_key: &str) -> Result<u64, errors::RedisError>;
    async fn get_many_views(&self, resource_key: &[String]) -> Result<HashMap<String, u64>, errors::RedisError>;
    async fn delete_interactions(&self, resource_key: &str) -> Result<(), errors::RedisError>;
    async fn delete_many_interactions(
        &self,
        resource_keys: &[String],
    ) -> Result<(), errors::RedisError>;
}

pub struct RedisInteractionRepository<C: CacheService + 'static> {
    cache_service: Arc<C>,
}

impl<C: CacheService + 'static> RedisInteractionRepository<C> {
    pub fn new(cache_service: Arc<C>) -> Self {
        Self { cache_service }
    }
}

#[async_trait]
impl<C: CacheService + 'static> InteractionRepository for RedisInteractionRepository<C> {
    async fn is_liked(
        &self,
        resource_key: &str,
        user_id: &Ulid,
    ) -> Result<bool, errors::RedisError> {
        let cache_key = format!("entity:{}:likes", resource_key);
        self.cache_service
            .sismember(&cache_key, &user_id.to_string())
            .await
    }

    async fn is_many_liked(
        &self,
        resource_keys: &[String],
        user_id: &Ulid,
    ) -> Result<HashMap<String, bool>, errors::RedisError> {
        if resource_keys.is_empty() { 
            return Ok(HashMap::new());
        }
        
        let cache_keys: Vec<String> = resource_keys
            .iter()
            .map(|key| format!("entity:{}:likes", key))
            .collect();
        
        let results = self.cache_service
            .sismember_many(&cache_keys, &user_id.to_string())
            .await?;

        let mut response = HashMap::new();
        for (cache_key, views) in results {
            if let Some(resource_key) = cache_key
                .strip_prefix("entity:")
                .and_then(|s| s.strip_suffix(":likes"))
            {
                response.insert(resource_key.to_string(), views);
            }
        }
        
        Ok(response)
    }

    async fn get_likes(&self, resource_key: &str) -> Result<u64, errors::RedisError> {
        let cache_key = format!("entity:{}:likes", resource_key);
        self.cache_service.scard(&cache_key).await
    }

    async fn get_many_likes(
        &self,
        resource_keys: &[String],
    ) -> Result<HashMap<String, u64>, errors::RedisError> {
        if resource_keys.is_empty() {
            return Ok(HashMap::new());
        }

        let cache_keys: Vec<String> = resource_keys
            .iter()
            .map(|key| format!("entity:{}:likes", key))
            .collect();
        
        let many_likes = self.cache_service.scard_many(&cache_keys).await?;
        let mut response = HashMap::new();

        for (cache_key, views) in many_likes {
            if let Some(resource_key) = cache_key
                .strip_prefix("entity:")
                .and_then(|s| s.strip_suffix(":likes"))
            {
                response.insert(resource_key.to_string(), views);
            }
        }
        
        Ok(response)
    }

    async fn like(&self, resource_key: &str, user_id: &Ulid) -> Result<bool, errors::RedisError> {
        let cache_key = format!("entity:{}:likes", resource_key);
        self.cache_service
            .sadd(&cache_key, &user_id.to_string())
            .await
    }

    async fn unlike(&self, resource_key: &str, user_id: &Ulid) -> Result<bool, errors::RedisError> {
        let cache_key = format!("entity:{}:likes", resource_key);
        self.cache_service
            .srem(&cache_key, &user_id.to_string())
            .await
    }

    async fn view(&self, resource_key: &str, user_id: &Ulid) -> Result<bool, errors::RedisError> {
        let cache_key = format!("entity:{}:views", resource_key);
        self.cache_service
            .pfadd(&cache_key, &user_id.to_string())
            .await
    }

    async fn get_views(&self, resource_key: &str) -> Result<u64, errors::RedisError> {
        let cache_key = format!("entity:{}:views", resource_key);
        let views = self
            .cache_service
            .pfcount(&cache_key)
            .await?;

        Ok(views)
    }

    async fn get_many_views(
        &self,
        resource_keys: &[String],
    ) -> Result<HashMap<String, u64>, errors::RedisError> {
        if resource_keys.is_empty() {
            return Ok(HashMap::new());
        }

        let cache_keys: Vec<String> = resource_keys
            .iter()
            .map(|key| format!("entity:{}:views", key))
            .collect();

        let many_views = self.cache_service.pfcount_many(&cache_keys).await?;
        let mut response = HashMap::new();
        
        for (cache_key, views) in many_views {
            if let Some(resource_key) = cache_key
                .strip_prefix("entity:")
                .and_then(|s| s.strip_suffix(":views"))
            {
                response.insert(resource_key.to_string(), views);
            }
            
        }
        Ok(response)
    }

    async fn delete_interactions(&self, resource_key: &str) -> Result<(), errors::RedisError> {
        let cache_key_views = format!("entity:{}:views", resource_key);
        let cache_key_likes = format!("entity:{}:likes", resource_key);

        self.cache_service
            .del(&[cache_key_views, cache_key_likes])
            .await
    }

    async fn delete_many_interactions(
        &self,
        resource_keys: &[String],
    ) -> Result<(), errors::RedisError> {
        if resource_keys.is_empty() { 
            return Ok(());
        }
        
        let mut cache_keys = Vec::new();
        for resource_key in resource_keys {
            let cache_key_views = format!("entity:{}:views", resource_key);
            let cache_key_likes = format!("entity:{}:likes", resource_key);

            cache_keys.push(cache_key_likes);
            cache_keys.push(cache_key_views);
        }

        self.cache_service.del(&cache_keys).await
    }
}
