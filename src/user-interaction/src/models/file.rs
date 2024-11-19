use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileMetadata {
    #[serde(rename="fileName")]
    pub file_name: String,
    #[serde(rename="contentType")]
    pub content_type: String,
    pub access_url: PresignedUrl,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PresignedUrl {
    pub url: String,
    pub expire_in: DateTime<Utc>
}