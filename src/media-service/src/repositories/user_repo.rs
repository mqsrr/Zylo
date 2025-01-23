use crate::errors;
use async_trait::async_trait;
use mongodb::bson::doc;
use mongodb::{ClientSession, Collection, Database};
use ulid::Ulid;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn start_session(&self) -> Result<ClientSession, errors::AppError>;
    async fn create(&self, user_id: &Ulid) -> Result<(), errors::AppError>;
    async fn exists(&self, user_id: &Ulid) -> Result<bool, errors::AppError>;
    async fn delete(
        &self,
        user_id: &Ulid,
        session: &mut ClientSession,
    ) -> Result<(), errors::AppError>;
}

pub struct MongoUserRepository {
    db: Database,
    collection: Collection<Ulid>,
}

impl MongoUserRepository {
    pub fn new(db: Database) -> Self {
        let collection = db.collection::<Ulid>("users");
        Self { db, collection }
    }
}

#[async_trait]
impl UserRepository for MongoUserRepository {
    async fn start_session(&self) -> Result<ClientSession, errors::AppError> {
        Ok(self.db
            .client()
            .start_session()
            .await
            .map_err(|err| errors::MongoError::DatabaseError(err.to_string()))?)
    }

    async fn create(&self, user_id: &Ulid) -> Result<(), errors::AppError> {
        self.collection
            .insert_one(user_id)
            .await
            .map_err(|err| errors::MongoError::DatabaseError(err.to_string()))?;

        Ok(())
    }

    async fn exists(&self, user_id: &Ulid) -> Result<bool, errors::AppError> {
        let filter = doc! { "_id": user_id.to_string() };
        let count = self
            .collection
            .count_documents(filter)
            .await
            .map_err(|err| errors::MongoError::DatabaseError(err.to_string()))?;

        Ok(count > 0)
    }

    async fn delete(
        &self,
        user_id: &Ulid,
        session: &mut ClientSession,
    ) -> Result<(), errors::AppError> {
        let filter = doc! { "_id": user_id.to_string() };
        let result = self
            .collection
            .delete_one(filter)
            .session(session)
            .await
            .map_err(|err| errors::MongoError::DatabaseError(err.to_string()))?;

        if result.deleted_count == 0 {
            return Err(errors::MongoError::NotFound("User with given id could not be found".to_string()))?;
        }

        Ok(())
    }
}
