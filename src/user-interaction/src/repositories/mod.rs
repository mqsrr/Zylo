use sqlx::{migrate, PgPool};
use sqlx::postgres::PgPoolOptions;
use crate::errors;
use crate::settings::Database;

pub mod reply_repo;
pub mod interaction_repo;
pub mod posts_repo;
pub mod users_repo;

pub async fn init_db(config: &Database) -> Result<PgPool, errors::DatabaseError> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.uri)
        .await
        .map_err(|e| errors::DatabaseError::PoolCreationError(e.to_string()))?;

    migrate!().run(&pool).await?;
    Ok(pool)
}