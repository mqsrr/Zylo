use crate::errors;
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use ulid::Ulid;

pub type DeletedPostIds = Vec<String>;

#[async_trait]
pub trait UsersRepository: Send + Sync {
    async fn create(&self, user_id: &Ulid) -> Result<(), errors::DatabaseError>;
    async fn delete(&self, user_id: &Ulid) -> Result<DeletedPostIds, errors::DatabaseError>;
}

pub struct PostgresUsersRepository {
    pool: PgPool,
}

impl PostgresUsersRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UsersRepository for PostgresUsersRepository {
    async fn create(&self, user_id: &Ulid) -> Result<(), errors::DatabaseError> {
        sqlx::query(
            r#"
            INSERT INTO users (id)
            VALUES ($1)
            "#,
        )
        .bind(user_id.to_bytes())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete(&self, user_id: &Ulid) -> Result<DeletedPostIds, errors::DatabaseError> {
        let mut transaction = self.pool.begin().await?;
        let rows = sqlx::query(
            r#"
                SELECT id
                FROM posts
                WHERE user_id = $1
            "#,
        )
        .bind(user_id.to_bytes())
        .fetch_all(&mut *transaction)
        .await?;

        let result = sqlx::query(
            r#"
                DELETE FROM users
                WHERE id = $1
            "#,
        )
        .bind(user_id.to_bytes())
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;

        if result.rows_affected() == 0 {
            return Err(errors::DatabaseError::NotFound(String::from(
                "User with given id has not been found",
            )));
        }

        let deleted_posts: Vec<String> = rows
            .into_iter()
            .map(|row| {
                let id: Vec<u8> = row.get("id");
                Ulid::from_bytes(id.try_into().unwrap()).to_string()
            })
            .collect();

        Ok(deleted_posts)
    }
}
