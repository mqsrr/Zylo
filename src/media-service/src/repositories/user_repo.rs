use crate::errors;
use async_trait::async_trait;
use mongodb::bson::{doc};
use mongodb::{Collection, Database};
use serde::Serialize;
use ulid::Ulid;

#[derive(Serialize)]
struct UserIdRow {
    _id: Ulid
}

#[async_trait]
pub trait UsersRepository: Send + Sync {
    async fn create(&self, user_id: Ulid) -> Result<(), errors::AppError>;
    async fn exists(&self, user_id: &Ulid) -> Result<bool, errors::AppError>;
    async fn delete(
        &self,
        user_id: &Ulid,
    ) -> Result<(), errors::AppError>;
}

pub struct MongoUserRepository {
    collection: Collection<UserIdRow>,
}

impl MongoUserRepository {
    pub fn new(db: Database) -> Self {
        Self { collection: db.collection::<UserIdRow>("users") }
    }
}

#[async_trait]
impl UsersRepository for MongoUserRepository {
    async fn create(&self, user_id: Ulid) -> Result<(), errors::AppError> {
        self.collection
            .insert_one(UserIdRow{_id: user_id})
            .await
            .map_err(errors::MongoError::DatabaseError)?;

        Ok(())
    }

    async fn exists(&self, user_id: &Ulid) -> Result<bool, errors::AppError> {
        let filter = doc! { "_id": user_id.to_string() };
        let count = self
            .collection
            .count_documents(filter)
            .await
            .map_err(errors::MongoError::DatabaseError)?;

        Ok(count > 0)
    }

    async fn delete(
        &self,
        user_id: &Ulid,
    ) -> Result<(), errors::AppError> {
        let filter = doc! { "_id": user_id.to_string() };
        let result = self
            .collection
            .delete_one(filter)
            .await
            .map_err(errors::MongoError::DatabaseError)?;

        if result.deleted_count == 0 {
            return Err(errors::MongoError::NotFound(
                "User with given id could not be found".to_string(),
            ))?;
        }

        Ok(())
    }
}
