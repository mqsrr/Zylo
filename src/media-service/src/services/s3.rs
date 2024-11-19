use crate::errors::AppError;
use crate::models::file::{File, PresignedUrl};
use crate::settings::S3Settings;
use async_trait::async_trait;
use aws_config::BehaviorVersion;
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;
use std::ops::Add;
use std::time::Duration;

#[async_trait]
pub trait S3Service {
    async fn upload(&self, file: &File) -> Result<PresignedUrl, AppError>;
    async fn delete(&self, key: &str) -> Result<(), AppError>;
    async fn get_presigned_url_for_download(&self, key: &str) -> Result<PresignedUrl, AppError>;
}

#[derive(Clone)]
pub struct S3FileService {
    pub client: Client,
    pub settings: S3Settings,
}

impl S3FileService {
    pub fn new(client: Client, settings: S3Settings) -> Self {
        Self { client, settings }
    }
}

#[async_trait]
impl S3Service for S3FileService {
    async fn upload(&self, file: &File) -> Result<PresignedUrl, AppError> {
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

        let url = self.get_presigned_url_for_download(key).await?;
        Ok(url)
    }

    async fn delete(&self, key: &str) -> Result<(), AppError> {
        self.client
            .delete_object()
            .bucket(&self.settings.bucket_name)
            .key(key)
            .send()
            .await
            .map_err(|e| e.into_service_error())?;

        Ok(())
    }

    async fn get_presigned_url_for_download(&self, key: &str) -> Result<PresignedUrl, AppError> {
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

pub async fn init_s3_client() -> Result<Client, AppError> {
    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    Ok(Client::new(&config))
}
