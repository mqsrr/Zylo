use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use crate::setting::{Database};

pub async fn init_db(config: &Database) -> PgPool {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.uri).await.expect("Failed to connect to database")
}