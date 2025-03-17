use std::fs;
use async_trait::async_trait;
use mongodb::bson::doc;
use mongodb::Database;
use crate::{errors, settings};

#[async_trait]
pub trait Finalizer {
    async fn finalize(&self) -> Result<(), errors::AppError>;
}

pub async fn init_db(config: &settings::Database) -> Database {
    let db_uri = config.uri.as_str();
    let db_name = config.name.as_str();

    let database = mongodb::Client::with_uri_str(db_uri)
        .await
        .expect("Failed to initialize MongoDB connection")
        .database(db_name);

    let ping_result = database.run_command(doc! {"ping": 1}).await;
    match ping_result {
        Ok(_) => database,
        Err(_) => panic!("Failed to ping"),
    }
}

pub fn get_container_id() -> Option<String> {
    if let Ok(cgroup) = fs::read_to_string("/proc/self/cgroup") {
        for line in cgroup.lines() {
            if let Some(id) = line.split('/').last() {
                if id.len() >= 12 {
                    return Some(id.to_string());
                }
            }
        }
    }
    None
}
