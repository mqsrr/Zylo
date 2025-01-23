use crate::errors;
use crate::models::file::{File, FileMetadata, PresignedUrl};
use crate::settings::S3Settings;
use async_trait::async_trait;
use aws_config::BehaviorVersion;
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;
use std::ops::Add;
use std::time::Duration;

#[async_trait]
pub trait S3Service: Send + Sync {
    async fn upload(&self, file: File) -> Result<FileMetadata, errors::S3Error>;
    async fn delete(&self, key: &str) -> Result<(), errors::S3Error>;
    async fn get_presigned_url(&self, key: &str) -> Result<PresignedUrl, errors::S3Error>;
}

#[derive(Clone)]
pub struct S3FileService {
    pub client: Client,
    pub settings: S3Settings,
}

impl S3FileService {
    pub async fn new(settings: S3Settings) -> Self {
        let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
        let client = Client::new(&config);
        
        Self { client, settings }
    }
}

#[async_trait]
impl S3Service for S3FileService {
    async fn upload(&self, file: File) -> Result<FileMetadata, errors::S3Error> {
        let byte_stream = ByteStream::from(file.content.clone());
        let key = &format!("media_images/{}", &file.id);
        self.client
            .put_object()
            .bucket(&self.settings.bucket_name)
            .key(key)
            .body(byte_stream)
            .content_type(&file.content_type)
            .metadata("file-name", &file.file_name)
            .send()
            .await
            .map_err(|e| e.into_service_error())?;

        let url = self.get_presigned_url(key).await?;
        
        Ok(FileMetadata {
            id: file.id,
            file_name: file.file_name,
            content_type: file.content_type,
            url: Some(url),
        })
    }

    async fn delete(&self, key: &str) -> Result<(), errors::S3Error> {
        self.client
            .delete_object()
            .bucket(&self.settings.bucket_name)
            .key(key)
            .send()
            .await
            .map_err(|e| e.into_service_error())?;

        Ok(())
    }

    async fn get_presigned_url(&self, key: &str) -> Result<PresignedUrl, errors::S3Error> {
        let expire = self.settings.expire_time * 60;
        let presigned_req = self
            .client
            .get_object()
            .bucket(&self.settings.bucket_name)
            .key(key)
            .presigned(PresigningConfig::expires_in(Duration::from_secs(expire as u64)).unwrap())
            .await
            .map_err(|e| e.into_service_error())?;

        let uri = presigned_req.uri().to_string();

        Ok(PresignedUrl {
            url: uri,
            expire_in: Some(chrono::Utc::now().add(Duration::from_secs(expire as u64))),
        })
    }
}