use crate::errors::AppError;
use crate::models::event_messages::{UserCreatedMessage, UserDeletedMessage, UserUpdatedMessage};
use crate::models::file::{FileMetadata, FileMetadataResponse};
use mongodb::bson::doc;
use mongodb::{Collection, Database};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use crate::services::user_profile::{ProfilePictureService, UserProfileService};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: Ulid,
    pub name: String,
    pub username: String,
    pub bio: String,
    pub location: String,
    pub profile_image: Option<FileMetadata>,
}

impl User {
    pub async fn create(message: UserCreatedMessage, db: &Database) -> Result<(), AppError> {
        let collection: Collection<Self> = db.collection("users");
        let user = Self::from_create_message(message);
        collection.insert_one(&user).await?;

        Ok(())
    }

    pub async fn get(user_id: Ulid, db: &Database, profile_client: &mut UserProfileService) -> Result<Self, AppError> {
        let collection: Collection<Self> = db.collection("users");
        let mut user: User = collection
            .find_one(doc! {"_id": user_id.to_string()})
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        user.profile_image = Some(profile_client.get_profile_picture(user_id).await?);
        Ok(user)
    }

    pub async fn update(message: UserUpdatedMessage, db: &Database) -> Result<(), AppError> {
        let collection: Collection<Self> = db.collection("users");

        let filter = doc! { "_id": message.id.to_string() };
        let update = doc! { "$set": doc! {
            "name": message.name,
            "bio": message.bio,
            "location": message.location,
        } };

        collection.update_one(filter, update).await?;
        Ok(())
    }

    pub async fn delete(message: UserDeletedMessage, db: &Database) -> Result<(), AppError> {
        let collection: Collection<Self> = db.collection("users");
        collection
            .delete_one(doc! { "_id": message.id.to_string()})
            .await?;

        Ok(())
    }
    
    pub fn from_create_message (message: UserCreatedMessage) -> Self {
        Self{
            id: message.id,
            name: message.name,
            username: message.username,
            bio: String::new(),
            location: String::new(),
            profile_image: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: Ulid,
    pub name: String,
    pub username: String,
    pub bio: String,
    pub location: String,
    #[serde(rename = "profileImage")]
    pub profile_image: FileMetadataResponse,
}

impl UserResponse {
    pub fn from(value: User) -> Self {
        Self {
            id: value.id,
            name: value.name,
            username: value.username,
            bio: value.bio,
            location: value.location,
            profile_image: value.profile_image.unwrap().into(),
        }
    }
}
