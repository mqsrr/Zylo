use crate::errors::AppError;
use crate::models::file::{File, FileMetadata, FileMetadataResponse};
use crate::models::user::{User, UserResponse};
use crate::services::s3::S3Service;
use crate::utils::requests::{CreatePostRequest, UpdatePostRequest};
use futures_util::TryStreamExt;
use log::warn;
use mongodb::bson::doc;
use mongodb::{Collection, Database};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use ulid::Ulid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    #[serde(rename = "_id")]
    pub id: Ulid,
    pub user_id: Ulid,
    pub text: String,
    pub files_metadata: Vec<FileMetadata>,
    pub created_at: String,
    pub updated_at: String,
}

impl Post {
    pub async fn create(
        mut request: CreatePostRequest,
        db: &Database,
        s3file_service: Arc<impl S3Service>,
    ) -> Result<Self, AppError> {
        let collection: Collection<Self> = db.collection("posts");
        File::process_files(&mut request.files, s3file_service).await?;

        let post = Self::from_request(request);
        collection.insert_one(&post).await?;

        Ok(post)
    }

    pub async fn update(
        mut request: UpdatePostRequest,
        db: &Database,
        s3file_service: Arc<impl S3Service>,
    ) -> Result<Self, AppError> {
        let collection: Collection<Self> = db.collection("posts");
        let post = Self::delete(request.id, db, s3file_service.clone()).await?;

        File::process_files(&mut request.files, s3file_service).await?;
        let post = Self::from_update_request(request, post.created_at);

        collection.insert_one(&post).await?;
        Ok(post)
    }

    pub async fn get(
        post_id: Ulid,
        db: &Database,
        s3file_service: Arc<impl S3Service>,
    ) -> Result<Self, AppError> {
        let collection: Collection<Self> = db.collection("posts");
        let mut post = collection
            .find_one(doc! {"_id": post_id.to_string()})
            .await?
            .ok_or_else(|| AppError::NotFound("Post not found".to_string()))?;

        File::populate_file_urls(&mut post.files_metadata, s3file_service).await?;
        Ok(post)
    }

    pub async fn get_posts(
        db: &Database,
        s3file_service: Arc<impl S3Service>,
        user_id: Option<Ulid>,
        last_created_at: Option<String>,
        per_page: Option<u16>,
    ) -> Result<Vec<Post>, AppError> {
        let collection: Collection<Self> = db.collection("posts");
        let per_page = per_page.unwrap_or(10);

        let mut filter_doc = doc! {};
        if let Some(user_id) = user_id {
            filter_doc.insert("user_id", user_id.to_string());
        }

        if let Some(last_created_at) = last_created_at {
            filter_doc.insert("created_at", doc! { "$lt": last_created_at });
        }

        let mut cursor = collection.find(filter_doc)
            .sort(doc! { "created_at": -1 })
            .limit(per_page as i64).
            await?;

        let mut posts = Vec::new();
        while let Some(mut post) = cursor.try_next().await? {
            File::populate_file_urls(&mut post.files_metadata, s3file_service.clone()).await?;
            posts.push(post);
        }

        Ok(posts)
    }

    pub async fn delete(
        post_id: Ulid,
        db: &Database,
        s3file_service: Arc<impl S3Service>,
    ) -> Result<Self, AppError> {
        let collection: Collection<Self> = db.collection("posts");
        let deleted_post = collection
            .find_one_and_delete(doc! {"_id": post_id.to_string()})
            .await?
            .ok_or_else(|| AppError::NotFound("Post not found".to_string()))?;

        FileMetadata::delete_files(&deleted_post.files_metadata, s3file_service).await?;
        Ok(deleted_post)
    }

    pub async fn delete_all_from_user(
        user_id: Ulid,
        db: &Database,
        s3file_service: Arc<impl S3Service>,
    ) -> Result<(), AppError> {
        let collection: Collection<Self> = db.collection("posts");

        let filter = doc! { "user_id": user_id.to_string() };
        let mut cursor = collection.find(filter.clone()).await?;

        let mut file_ids_to_delete = Vec::new();
        while let Some(post) = cursor.try_next().await? {
            file_ids_to_delete.extend(post.files_metadata.into_iter().map(|file| file.id));
        }

        collection.delete_many(filter).await?;
        for file_id in file_ids_to_delete {
            if let Err(err) = s3file_service
                .delete(&format!("media_images/{}", file_id))
                .await
            {
                warn!("Error deleting file: {:?}", err);
            }
        }

        Ok(())
    }
    fn from_request(request: CreatePostRequest) -> Self {
        Self {
            id: request.id,
            user_id: request.user_id,
            text: request.text.clone(),
            files_metadata: request.files.into_iter().map(FileMetadata::from).collect(),
            created_at: request.created_at,
            updated_at: request.updated_at,
        }
    }

    fn from_update_request(request: UpdatePostRequest, created_at: String) -> Self {
        Self {
            id: request.id,
            user_id: request.user_id,
            text: request.text.clone(),
            files_metadata: request.files.into_iter().map(FileMetadata::from).collect(),
            created_at,
            updated_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostResponse {
    pub id: String,
    pub user: UserResponse,
    pub text: String,
    #[serde(rename = "filesMetadata")]
    pub files_metadata: Vec<FileMetadataResponse>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

impl PostResponse {
    pub fn from(value: Post, user: User) -> Self {
        Self {
            id: value.id.to_string(),
            user: UserResponse::from(user),
            text: value.text,
            files_metadata: value
                .files_metadata
                .into_iter()
                .map(FileMetadataResponse::from)
                .collect(),
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
