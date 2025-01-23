use crate::errors;
use crate::models::post::{DeletedPostsIds, Post};
use crate::services::s3_service::S3Service;
use crate::utils::request::{CreatePostRequest, PaginatedResponse, UpdatePostRequest};
use async_trait::async_trait;
use futures_util::TryStreamExt;
use mongodb::bson::doc;
use mongodb::{ClientSession, Collection, Database};
use std::sync::Arc;
use ulid::Ulid;

#[async_trait]
pub trait PostRepository: Send + Sync {
    async fn create(&self, post: CreatePostRequest) -> Result<Post, errors::AppError>;
    async fn update(&self, post: UpdatePostRequest) -> Result<Post, errors::AppError>;
    async fn get(&self, post_id: &Ulid) -> Result<Post, errors::AppError>;
    async fn get_paginated_posts(
        &self,
        user_id: Option<Ulid>,
        per_page: Option<u16>,
        last_post_id: Option<Ulid>,
    ) -> Result<PaginatedResponse<Post>, errors::AppError>;
    async fn get_batch_posts(&self, post_ids: Vec<Ulid>) -> Result<Vec<Post>, errors::AppError>; 
    async fn delete(&self, post_id: &Ulid) -> Result<(), errors::AppError>;
    async fn delete_all_from_user(&self, user_id: &Ulid, session: &mut ClientSession) -> Result<DeletedPostsIds, errors::AppError>;
}

#[derive(Debug, Clone)]
pub struct MongoPostRepository<S: S3Service + 'static> {
    collection: Collection<Post>,
    s3_service: Arc<S>
}

impl<S: S3Service + 'static> MongoPostRepository<S> {
    pub fn new(db: &Database, s3_service: Arc<S>) -> Self {
        Self {
            collection: db.collection("posts"),
            s3_service
        }
    }
    
    async fn attach_presigned_urls(&self, posts: &mut [Post]) -> Result<(), errors::AppError> {
        for post in posts {
            for file_metadata in &mut post.files_metadata {
                file_metadata.url = Some(
                    self.s3_service
                        .get_presigned_url(&format!("media_images/{}", file_metadata.id))
                        .await?,
                );
            }
        }
        Ok(())
    }
}

#[async_trait]
impl<S: S3Service + 'static> PostRepository for MongoPostRepository<S>{
    async fn create(&self, request: CreatePostRequest) -> Result<Post, errors::AppError> {
        let files = request.files.clone();
        let post = Post::from(request);
        
        self.collection
            .insert_one(&post)
            .await
            .map_err(|err| errors::MongoError::DatabaseError(err.to_string()))?;

        for file in files {
            self.s3_service.upload(file).await?;
        }
        
        Ok(post)
    }

    async fn update(&self, request: UpdatePostRequest) -> Result<Post, errors::AppError> {
        let update = self.collection
            .find_one_and_update(doc! {"_id": request.id.to_string()}, doc! {
                "text": request.text.to_string()
            })
            .await
            .map_err(|err| errors::MongoError::DatabaseError(err.to_string()))?;

        for file in request.files {
            self.s3_service.upload(file).await?;
        }
        
        Ok(update.unwrap())
    }

    async fn get(&self, post_id: &Ulid) -> Result<Post, errors::AppError> {
        let mut post = self
            .collection
            .find_one(doc! {"_id": post_id.to_string()})
            .await
            .map_err(|err| errors::MongoError::DatabaseError(err.to_string()))?
            .ok_or_else(|| errors::MongoError::NotFound("Post with given id does not exists".to_string()))?;


        for file_metadata in &mut post.files_metadata {
            file_metadata.url = Some(
                self.s3_service
                    .get_presigned_url(&format!("media_images/{}", file_metadata.id))
                    .await?,
            );
        }
        Ok(post)
    }

    async fn get_paginated_posts(
        &self,
        user_id: Option<Ulid>,
        per_page: Option<u16>,
        last_post_id: Option<Ulid>,
    ) -> Result<PaginatedResponse<Post>, errors::AppError> {
        let per_page = per_page.unwrap_or(10);
        let mut filter_doc = doc! {};

        if let Some(user_id) = user_id {
            filter_doc.insert("user_id", user_id.to_string());
        }

        if let Some(last_post_id) = last_post_id {
            filter_doc.insert("_id", doc! { "$lt": last_post_id.to_string() });
        }

        let mut cursor = self
            .collection
            .find(filter_doc)
            .sort(doc! { "_id": -1 })
            .limit((per_page + 1) as i64)
            .await
            .map_err(|err| errors::MongoError::DatabaseError(err.to_string()))?;

        let mut posts = Vec::new();
        while let Some(post) = cursor
            .try_next()
            .await
            .map_err(|err| errors::MongoError::DatabaseError(err.to_string()))?
        {
            posts.push(post);
        }

        self.attach_presigned_urls(&mut posts).await?;
        let next_cursor = posts
            .len()
            .checked_sub(1)
            .filter(|&len| len >= per_page as usize) 
            .and_then(|_| posts.pop()) 
            .map(|post| post.id.to_string());
        
        let has_next_page = next_cursor.is_some();
        Ok(PaginatedResponse::new(posts, per_page, has_next_page, next_cursor.unwrap_or_default()))
    }

    async fn get_batch_posts(&self, post_ids: Vec<Ulid>) -> Result<Vec<Post>, errors::AppError> {
        if post_ids.is_empty() {
            return Ok(vec![]);
        }

        let filter_doc = doc! {
            "_id": { "$in": post_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>() }
        };

        let mut cursor = self
            .collection
            .find(filter_doc)
            .await
            .map_err(|err| errors::MongoError::DatabaseError(err.to_string()))?;

        let mut posts = Vec::new();
        while let Some(post) = cursor
            .try_next()
            .await
            .map_err(|err| errors::MongoError::DatabaseError(err.to_string()))?
        {
            posts.push(post);
        }

        self.attach_presigned_urls(&mut posts).await?;
        Ok(posts)
    }

    async fn delete(&self, post_id: &Ulid) -> Result<(), errors::AppError> {
        let deleted_post = self
            .collection
            .find_one_and_delete(doc! {"_id": post_id.to_string()})
            .await
            .map_err(|err| errors::MongoError::DatabaseError(err.to_string()))?
            .ok_or_else(|| errors::MongoError::NotFound("Post with given id could not be found".to_string()))?;
        
        for file_metadata in &deleted_post.files_metadata {
            self.s3_service.delete(&format!("media_images/{}", file_metadata.id)).await?;
        }
        
        Ok(())
    }

    async fn delete_all_from_user(&self, user_id: &Ulid, session: &mut ClientSession) -> Result<DeletedPostsIds, errors::AppError> {
        let filter = doc! { "user_id": user_id.to_string() };
        let mut cursor = self
            .collection
            .find(filter.clone())
            .await
            .map_err(|err| errors::MongoError::DatabaseError(err.to_string()))?;
        
        self.collection
            .delete_many(filter)
            .session(session)
            .await
            .map_err(|err| errors::MongoError::DatabaseError(err.to_string()))?;

        let mut deleted_posts_ids = Vec::new();
        while let Some(post) = cursor
            .try_next()
            .await
            .map_err(|err| errors::MongoError::DatabaseError(err.to_string()))?
        {
            deleted_posts_ids.push(post.id);
            for file_metadata in post.files_metadata {
                self.s3_service.delete(&format!("media_images/{}", file_metadata.id)).await?;
            }
        }
        
        Ok(deleted_posts_ids)
    }
}
