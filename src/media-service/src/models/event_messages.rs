use crate::models::post::Post;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Debug, Serialize)]
pub struct PostCreatedMessage {
    pub id: Ulid,
    #[serde(rename = "userId")]
    pub user_id: Ulid,
    pub content: String,
    #[serde(rename = "fileIds")]
    pub file_ids: Vec<Ulid>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

impl From<&Post> for PostCreatedMessage {
    fn from(value: &Post) -> Self {
        Self {
            id: value.id.clone(),
            user_id: value.user_id.clone(),
            content: value.text.clone(),
            file_ids: value.files_metadata.iter().map(|f| f.id).collect(),
            created_at: value.created_at.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PostUpdatedMessage {
    pub id: Ulid,
    pub content: String,
    #[serde(rename = "fileIds")]
    pub file_ids: Vec<Ulid>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

impl From<&Post> for PostUpdatedMessage {
    fn from(value: &Post) -> Self {
        Self {
            id: value.id.clone(),
            content: value.text.clone(),
            file_ids: value.files_metadata.iter().map(|f| f.id).collect(),
            created_at: value.created_at.clone(),
            updated_at: value.updated_at.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PostDeletedMessage {
    pub id: Ulid,
    #[serde(rename = "userId")]
    pub user_id: Ulid,
}

impl PostDeletedMessage {
    pub fn new(id: Ulid, user_id: Ulid) -> Self {
        Self { id, user_id }
    }
}

#[derive(Debug, Deserialize)]
pub struct UserDeletedMessage {
    pub id: Ulid,
}

#[derive(Debug, Deserialize)]
pub struct UserUpdatedMessage {
    pub id: Ulid,
    pub name: String,
    pub bio: String,
    pub location: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserCreatedMessage {
    pub id: Ulid,
    pub name: String,
    pub username: String,
}
