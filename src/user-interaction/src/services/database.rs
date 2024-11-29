use sqlx::{migrate, PgPool};
use sqlx::postgres::PgPoolOptions;
use crate::setting::{Database};

pub async fn init_db(config: &Database) -> PgPool {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.uri).await.expect("Failed to connect to database");
    
    migrate!()
        .run(&pool)
        .await
        .expect("Failed to migrate");

    pool
}