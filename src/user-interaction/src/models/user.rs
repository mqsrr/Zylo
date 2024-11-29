use serde::{Deserialize, Serialize};
use ulid::Ulid;
use crate::models::file::{FileMetadata, FileMetadataResponse};

#[derive(Debug, Clone)]
pub struct User {
    pub id: Ulid,
    pub username: String,
    pub name: String,
    pub bio: Option<String>,
    pub location: Option<String>,
    pub profile_image: Option<FileMetadata>,
}

impl User {
    pub fn new (id: Ulid) -> Self {
        Self {
            id,
            username: String::new(),
            name: String::new(),
            bio: None,
            location: None,
            profile_image: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserResponse {
    pub id: Ulid,
    pub username: String,
    pub name: String,
    pub bio: String,
    pub location: String,
    #[serde(rename="profileImage")]
    pub profile_image: FileMetadataResponse,
}

impl From<User> for UserResponse{
    fn from(value: User) -> Self {
        Self {
            id: value.id,
            username: value.username,
            name: value.name,
            bio: value.bio.unwrap_or_default(),
            location: value.location.unwrap_or_default(),
            profile_image: value.profile_image.unwrap().into(),
        }
    }
}