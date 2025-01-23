use crate::errors;
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
}

impl File {
    pub async fn from_field(field: Field<'_>) -> Result<Self, errors::AppError> {
        let file_name = field.file_name().unwrap_or("unknown").to_string();
        let content_type = field.content_type().unwrap_or("").to_string();
        let content = field
            .bytes()
            .await
            .map_err(|_|  errors::AppError::BadRequest("lskdjf".to_string()))?;

        Ok(Self {
            id: Ulid::new(),
            file_name,
            content_type,
            content,
        })
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PresignedUrl {
    pub url: String,
    pub expire_in: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub id: Ulid,
    pub file_name: String,
    pub content_type: String,
    pub url: Option<PresignedUrl>,
}

impl From<File> for FileMetadata {
    fn from(file: File) -> Self {
        Self {
            id: file.id,
            file_name: file.file_name,
            content_type: file.content_type,
            url: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadataResponse {
    #[serde(rename = "fileName")]
    pub file_name: String,
    #[serde(rename = "contentType")]
    pub content_type: String,
    #[serde(rename = "url")]
    pub url: String,
}

impl From<FileMetadata> for FileMetadataResponse {
    fn from(file: FileMetadata) -> Self {
        Self {
            file_name: file.file_name,
            content_type: file.content_type,
            url: file.url.unwrap_or_default().url,
        }
    }
}