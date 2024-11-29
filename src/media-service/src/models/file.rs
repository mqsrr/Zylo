use crate::services::s3::S3Service;
use crate::{errors::AppError, services::s3::S3FileService};
use axum::extract::multipart::Field;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Debug, Clone, Deserialize)]
pub struct File {
    pub id: Ulid,
    pub file_name: String,
    pub content_type: String,
    pub content: Bytes,
    pub access_url: Option<PresignedUrl>,
}

impl File {
    pub async fn from_field(field: Field<'_>) -> Result<Self, AppError> {
        let file_name = field.file_name().unwrap_or("unknown").to_string();
        let content_type = field.content_type().unwrap_or("").to_string();
        let content = field
            .bytes()
            .await
            .map_err(|_| AppError::InvalidMultipartContent)?;

        Ok(Self {
            id: Ulid::new(),
            file_name,
            content_type,
            content,
            access_url: None,
        })
    }

    pub async fn process_files(
        files: &mut [File],
        s3file_service: &S3FileService,
    ) -> Result<(), AppError> {
        for file in files {
            file.access_url = Some(s3file_service.upload(file).await?);
        }
        Ok(())
    }

    pub async fn populate_file_urls(
        files_metadata: &mut [FileMetadata],
        s3file_service: &S3FileService,
    ) -> Result<(), AppError> {
        for file_metadata in files_metadata {
            file_metadata.access_url = s3file_service
                .get_presigned_url_for_download(&format!(
                    "media_images/{}",
                    file_metadata.id.to_string()
                ))
                .await?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PresignedUrl {
    pub url: String,
    pub expire_in: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub id: Ulid,
    pub file_name: String,
    pub content_type: String,
    pub access_url: PresignedUrl,
}

impl From<File> for FileMetadata {
    fn from(file: File) -> Self {
        Self {
            id: file.id,
            file_name: file.file_name,
            content_type: file.content_type,
            access_url: file.access_url.unwrap(),
        }
    }
}

impl FileMetadata {
    pub async fn delete_files(
        files_metadata: &[FileMetadata],
        s3file_service: &S3FileService,
    ) -> Result<(), AppError> {
        for file_metadata in files_metadata {
            s3file_service
                .delete(&format!("media_images/{}", file_metadata.id))
                .await?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadataResponse {
    #[serde(rename = "fileName")]
    pub file_name: String,
    #[serde(rename = "contentType")]
    pub content_type: String,
    pub url: String,
}

impl From<FileMetadata> for FileMetadataResponse {
    fn from(value: FileMetadata) -> Self {
        Self {
            file_name: value.file_name,
            content_type: value.content_type,
            url: value.access_url.url,
        }
    }
}
