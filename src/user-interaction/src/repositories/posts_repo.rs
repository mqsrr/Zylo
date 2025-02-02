use crate::errors;
use async_trait::async_trait;
use sqlx::{PgPool};
use ulid::Ulid;


#[async_trait]
pub trait PostsRepository: Send + Sync {
    async fn create(&self, post_id: &Ulid, user_id: &Ulid) -> Result<(), errors::DatabaseError>;
    async fn delete(&self, post_id: &Ulid) -> Result<(),errors::DatabaseError>;
}

pub struct PostgresPostsRepository {
    pool: PgPool,
}

impl PostgresPostsRepository {
    pub fn new(pool: PgPool) -> Self{
        Self { pool }
    }
}

#[async_trait]
impl PostsRepository for PostgresPostsRepository {
    async fn create(&self, post_id: &Ulid, user_id: &Ulid) -> Result<(), errors::DatabaseError> {
        sqlx::query(
            r#"
            INSERT INTO posts (id, user_id)
            VALUES ($1, $2)
            "#,
        )
        .bind(post_id.to_bytes())
        .bind(user_id.to_bytes())
        .execute(&self.pool)
        .await?;

        Ok(())
    }
    
    async fn delete(&self, post_id: &Ulid) -> Result<(), errors::DatabaseError> {
        let result = sqlx::query(
            r#"
                DELETE FROM posts
                WHERE id = $1
            "#,
        )
        .bind(post_id.to_bytes())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(errors::DatabaseError::NotFound(String::from(
                "Post with given id has not been found",
            )));
        }
      
        Ok(())
    }
}
