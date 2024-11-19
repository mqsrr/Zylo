use crate::settings;
use mongodb::bson::doc;
use mongodb::Database;

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
