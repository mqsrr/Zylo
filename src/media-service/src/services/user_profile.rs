pub mod user_profile {
    tonic::include_proto!("user_profile");
}

use crate::errors::AppError;
use crate::models::file::{FileMetadata, PresignedUrl};
use chrono::{DateTime, Utc};
use async_trait::async_trait;
use ulid::Ulid;
use user_profile::user_profile_service_client::UserProfileServiceClient;
use user_profile::UserProfileRequest;

#[async_trait]
pub trait ProfilePictureService {
    async fn get_profile_picture(&mut self, user_id: Ulid) -> Result<FileMetadata, AppError>;
}

#[derive(Clone)]
pub struct UserProfileService {
    redis: redis::Client,
    client: UserProfileServiceClient<tonic::transport::Channel>,
}

#[async_trait]
impl ProfilePictureService for UserProfileService {
    async fn get_profile_picture(&mut self, user_id: Ulid) -> Result<FileMetadata, AppError> {
        if let Some(file) = crate::services::redis::hash_get(&self.redis, "media", &format!("profile_pictures/{}", user_id)).await? {
            return Ok(file);
        }

        let request = tonic::Request::new(UserProfileRequest {
            user_id: user_id.to_string(),
        });

        let response = self.client.get_profile_picture(request).await?;
        let user_profile_picture = response.get_ref();
        let expires_in = DateTime::<Utc>::from_timestamp_millis(user_profile_picture.expires_in).unwrap();
        
        let file = FileMetadata {
            id: user_id,
            file_name: user_profile_picture.file_name.clone(),
            content_type: user_profile_picture.content_type.clone(),
            access_url: PresignedUrl {
                url: user_profile_picture.profile_picture_url.clone(),
                expire_in: Some(expires_in),
            },
        };
        
        crate::services::redis::hash_set(&self.redis, "media", &format!("profile_pictures/{}", user_id), &file, (expires_in - Utc::now()).num_seconds()).await?;
        Ok(file)
    }
}

impl UserProfileService {
    pub fn new(redis: redis::Client, client: UserProfileServiceClient<tonic::transport::Channel>) -> Self {
        Self {
            redis,
            client,
        }
    }
}