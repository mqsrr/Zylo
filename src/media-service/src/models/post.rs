use crate::models::file::{FileMetadata, FileMetadataResponse};
use crate::utils::request::{CreatePostRequest};
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
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

pub type DeletedPostsIds = Vec<Ulid>;

impl From<CreatePostRequest> for Post {
    fn from(value: CreatePostRequest) -> Self {
        Self {
            id: Ulid::new(),
            user_id: value.user_id,
            text: value.text.clone(),
            files_metadata: value.files.into_iter().map(FileMetadata::from).collect(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostResponse {
    pub id: String,
    #[serde(rename = "userId")]
    pub user_id: String,
    pub text: String,
    #[serde(rename = "filesMetadata")]
    pub files_metadata: Vec<FileMetadataResponse>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

impl From<Post> for PostResponse {
    fn from(value: Post) -> Self {
        Self {
            id: value.id.to_string(),
            user_id: value.user_id.to_string(),
            text: value.text,
            files_metadata: value.files_metadata.into_iter().map(FileMetadataResponse::from).collect(),
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
