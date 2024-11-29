use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileMetadata {
    pub file_name: String,
    pub content_type: String,
    pub access_url: PresignedUrl,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PresignedUrl {
    pub url: String,
    pub expire_in: DateTime<Utc>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileMetadataResponse {
    #[serde(rename="fileName")]
    pub file_name: String,
    #[serde(rename="contentType")]
    pub content_type: String,
    pub url: String,
}

impl From<FileMetadata> for FileMetadataResponse {
    fn from(value: FileMetadata) -> Self {
        Self{
            url: value.access_url.url,
            file_name: value.file_name,
            content_type: value.content_type,
        }
    }
}