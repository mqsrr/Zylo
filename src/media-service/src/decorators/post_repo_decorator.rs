use crate::decorators::trace_server_error;
use crate::errors;
use crate::errors::AppError;
use crate::models::post::{DeletedPostsIds, Post};
use crate::repositories::post_repo::{MongoPostRepository, PostRepository};
use crate::services::cache_service::CacheService;
use crate::services::s3_service::S3Service;
use crate::utils::request::{CreatePostRequest, PaginatedResponse, UpdatePostRequest};
use async_trait::async_trait;
use mongodb::Database;
use opentelemetry::metrics::{Counter, Histogram};
use opentelemetry::{global, KeyValue};
use std::sync::Arc;
use tokio::time::Instant;
use tracing::{field, info_span};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use ulid::Ulid;

pub struct DecoratedPostRepositoryBuilder<P: PostRepository> {
    post_repo: P,
}

impl<S: S3Service + 'static> DecoratedPostRepositoryBuilder<MongoPostRepository<S>> {
    pub fn new(db: &Database, s3_service: Arc<S>) -> Self {
        Self {
            post_repo: MongoPostRepository::new(db, s3_service),
        }
    }
}

impl<P: PostRepository + 'static> DecoratedPostRepositoryBuilder<P> {
    pub fn cached<C: CacheService>(
        self,
        cache_service: Arc<C>,
    ) -> DecoratedPostRepositoryBuilder<CachedPostRepository<P, C>> {
        DecoratedPostRepositoryBuilder {
            post_repo: CachedPostRepository::new(self.post_repo, cache_service),
        }
    }
    pub fn observable(self) -> DecoratedPostRepositoryBuilder<ObservablePostRepository<P>> {
        DecoratedPostRepositoryBuilder {
            post_repo: ObservablePostRepository::new(self.post_repo),
        }
    }

    pub fn build(self) -> P {
        self.post_repo
    }
}

#[derive(Clone)]
pub struct ObservablePostRepository<P: PostRepository> {
    inner: P,
    request_count: Counter<u64>,
    request_latency: Histogram<f64>,
}

impl<P: PostRepository> ObservablePostRepository<P> {
    pub fn new(inner: P) -> Self {
        let boundaries = vec![
            0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
        ];

        let meter_provider = global::meter("media-service");
        let request_count = meter_provider
            .u64_counter("posts_repo_requests_total")
            .with_description("Total requests to PostsRepository")
            .with_unit("PostsRepository")
            .build();

        let request_latency = meter_provider
            .f64_histogram("posts_repo_request_duration_seconds")
            .with_description("Latency of PostsRepository methods")
            .with_boundaries(boundaries)
            .with_unit("PostsRepository")
            .build();

        Self {
            inner,
            request_count,
            request_latency,
        }
    }

    async fn track_method<T, F, E: errors::ProblemResponse + ToString>(
        &self,
        method_name: &str,
        query_summary: &str,
        operation_name: &str,
        target: &str,
        post_id: Option<&str>,
        operation: F,
    ) -> Result<T, E>
    where
        F: std::future::Future<Output = Result<T, E>>,
    {
        let start_time = Instant::now();
        let span = info_span!(
            "",
            "otel.name" = query_summary,
            "otel.kind" = "client",
            "db.system" = "mongodb",
            "db.operation" = operation_name,
            "db.target" = target,
            "method.name" = method_name,
            "post_id" = post_id,
            "error.message" = field::Empty,
            "error.type" = field::Empty,
        );

        let result = operation.await;
        let status = if result.is_ok() { "success" } else { "error" };
        let mut attributes = vec![
            KeyValue::new("method", method_name.to_string()),
            KeyValue::new("operation", operation_name.to_string()),
        ];

        self.request_latency
            .record(start_time.elapsed().as_secs_f64(), &attributes);

        attributes.push(KeyValue::new("status", status));
        self.request_count.add(1, &attributes);

        let result = trace_server_error(result, &span, "database_error")?;
        span.set_status(opentelemetry::trace::Status::Ok);

        Ok(result)
    }
}

#[async_trait]
impl<P: PostRepository + 'static> PostRepository for ObservablePostRepository<P> {
    async fn create(&self, post: CreatePostRequest) -> Result<Post, AppError> {
        self.track_method(
            "create",
            "mongo.insert_one posts",
            "insert_one",
            "posts",
            None,
            self.inner.create(post),
        )
        .await
    }

    async fn update(&self, post: UpdatePostRequest) -> Result<Post, AppError> {
        self.track_method(
            "update",
            "mongo.find_one_and_update posts",
            "find_one_and_update filter",
            "posts",
            Some(&post.id.to_string()),
            self.inner.update(post),
        )
        .await
    }

    async fn get(&self, post_id: &Ulid) -> Result<Post, AppError> {
        self.track_method(
            "get",
            "mongo.find_one posts",
            "find_one",
            "posts",
            Some(&post_id.to_string()),
            self.inner.get(post_id),
        )
        .await
    }

    async fn get_paginated_posts(
        &self,
        user_id: Option<Ulid>,
        per_page: Option<u32>,
        last_post_id: Option<Ulid>,
    ) -> Result<PaginatedResponse<Post>, AppError> {
        self.track_method(
            "get_paginated_posts",
            "mongo.find posts",
            "find, sort, count_documents",
            "posts",
            None,
            self.inner
                .get_paginated_posts(user_id, per_page, last_post_id),
        )
        .await
    }

    async fn get_batch_posts(&self, post_ids: Vec<Ulid>) -> Result<Vec<Post>, AppError> {
        self.track_method(
            "get_batch_posts",
            "mongo.find posts",
            "find",
            "posts",
            Some(
                &post_ids
                    .iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<_>>()
                    .join(" "),
            ),
            self.inner.get_batch_posts(post_ids),
        )
        .await
    }

    async fn delete(&self, post_id: &Ulid) -> Result<(), AppError> {
        self.track_method(
            "delete",
            "mongo.find_one_and_delete posts",
            "find_one_and_delete",
            "posts",
            Some(&post_id.to_string()),
            self.inner.delete(post_id),
        )
        .await
    }

    async fn delete_all_from_user(&self, user_id: &Ulid) -> Result<DeletedPostsIds, AppError> {
        self.track_method(
            "delete_all_from_user",
            "mongo.find.projection.delete_many posts",
            "find,projection,delete_many",
            "posts",
            None,
            self.inner.delete_all_from_user(user_id),
        )
        .await
    }
}

pub struct CachedPostRepository<P: PostRepository, C: CacheService> {
    inner: P,
    cache_service: Arc<C>,
}

impl<P: PostRepository, C: CacheService> CachedPostRepository<P, C> {
    pub fn new(inner: P, cache_service: Arc<C>) -> Self {
        Self {
            inner,
            cache_service,
        }
    }
}

#[async_trait]
impl<P: PostRepository, C: CacheService> PostRepository for CachedPostRepository<P, C> {
    async fn create(&self, post: CreatePostRequest) -> Result<Post, AppError> {
        let post = self.inner.create(post).await?;
        self.cache_service
            .hdelete_all("users-posts", &format!("*{}*", post.user_id))
            .await?;

        self.cache_service
            .hdelete_all("users-posts", "none:none:*")
            .await?;

        Ok(post)
    }

    async fn update(&self, post: UpdatePostRequest) -> Result<Post, AppError> {
        let updated_post = self.inner.update(post).await?;

        self.cache_service
            .hdelete_all("users-posts", &format!("*{}*", updated_post.user_id))
            .await?;

        self.cache_service
            .hdelete_all("users-posts", "none:none:*")
            .await?;

        self.cache_service
            .hdelete("posts", &updated_post.id.to_string())
            .await?;

        Ok(updated_post)
    }

    async fn get(&self, post_id: &Ulid) -> Result<Post, AppError> {
        let cache_key = &post_id.to_string();
        if let Some(post) = self.cache_service.hget::<Post>("posts", &cache_key).await? {
            return Ok(post);
        }

        let post = self.inner.get(post_id).await?;
        self.cache_service.hset("posts", cache_key, &post).await?;

        Ok(post)
    }

    async fn get_paginated_posts(
        &self,
        user_id: Option<Ulid>,
        per_page: Option<u32>,
        last_post_id: Option<Ulid>,
    ) -> Result<PaginatedResponse<Post>, AppError> {
        let user_id_str = user_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| "none".to_string());

        let last_post_id_str = last_post_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| "none".to_string());

        let cache_key = format!(
            "{}:{}:{}",
            &user_id_str,
            &last_post_id_str,
            &per_page.unwrap_or(10)
        );

        if let Some(paginated_posts) = self
            .cache_service
            .hget::<PaginatedResponse<Post>>("users-posts", &cache_key)
            .await?
        {
            return Ok(paginated_posts);
        }

        let paginated_posts = self
            .inner
            .get_paginated_posts(user_id, per_page, last_post_id)
            .await?;

        self.cache_service
            .hset("users-posts", &cache_key, &paginated_posts)
            .await?;

        Ok(paginated_posts)
    }

    async fn get_batch_posts(&self, post_ids: Vec<Ulid>) -> Result<Vec<Post>, AppError> {
        let cache_key = post_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<String>>()
            .join("-")
            .to_string();

        if let Some(cached_posts) = self
            .cache_service
            .hget::<Vec<Post>>("batch-posts", &cache_key)
            .await?
        {
            return Ok(cached_posts);
        }

        let posts = self.inner.get_batch_posts(post_ids).await?;

        self.cache_service
            .hset("batch-posts", &cache_key, &posts)
            .await?;

        Ok(posts)
    }

    async fn delete(&self, post_id: &Ulid) -> Result<(), AppError> {
        self.inner.delete(post_id).await?;

        self.cache_service
            .hdelete_all("batch-posts", &format!("*{}*", post_id))
            .await?;

        self.cache_service
            .hdelete_all("users-posts", "none:none:*")
            .await?;

        self.cache_service
            .hdelete("posts", &post_id.to_string())
            .await?;

        Ok(())
    }

    async fn delete_all_from_user(&self, user_id: &Ulid) -> Result<DeletedPostsIds, AppError> {
        let deleted_post_ids = self.inner.delete_all_from_user(user_id).await?;
        for post_id in &deleted_post_ids {
            self.cache_service
                .hdelete_all("batch-posts", &format!("*{}*", post_id))
                .await?;

            self.cache_service
                .hdelete_all("users-posts", &format!("*{}*", user_id))
                .await?;

            self.cache_service
                .hdelete_all("users-posts", "none:none:*")
                .await?;

            self.cache_service
                .hdelete("posts", &post_id.to_string())
                .await?;
        }

        Ok(deleted_post_ids)
    }
}
