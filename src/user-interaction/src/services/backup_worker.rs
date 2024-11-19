use std::collections::HashMap;
use log::{error, info};
use redis::{AsyncCommands, Client};
use tokio_cron_scheduler::{Job, JobScheduler};
use crate::errors::AppError;
use crate::setting::Redis;

pub async fn start_worker(redis_config: &Redis) -> Result<(), AppError> {
    let sched = JobScheduler::new().await?;
    let source_client_uri = redis_config.uri.clone();
    let backup_client_uri = redis_config.backup_uri.clone();

    let job = Job::new_cron_job_async("0 */10 * * * *", move |_uuid, _l| {
        Box::pin({
            {
                info!("Starting backup of Redis data");
                let cloned_source = source_client_uri.clone();
                let cloned_backup = backup_client_uri.clone();
                async move {
                    let source_client = Client::open(cloned_source).unwrap();
                    let backup_client = Client::open(cloned_backup).unwrap();

                    match save_data(source_client, backup_client).await {
                        Ok(_) => info!("Backup completed successfully."),
                        Err(e) => error!("Backup failed: {:?}", e),
                    }
                }
            }
        })
    })?;

    sched.add(job).await?;
    sched.shutdown_on_ctrl_c();
    
    sched.start().await?;
    Ok(())
}

async fn fetch_all_hashsets(client: &Client, key: &str) -> Result<HashMap<String, i32>, AppError> {
    let mut conn = client.get_multiplexed_async_connection().await?;
    let data: HashMap<String, String> = conn.hgetall(key).await?;

    let mut hashset_data: HashMap<String, i32> = HashMap::new();
    for (field, value) in data.into_iter() {
        let count = value.parse::<i32>().unwrap_or(0);
        hashset_data.insert(field, count);
    }

    Ok(hashset_data)
}

async fn fetch_all_sets(client: &Client, key: &str) -> Result<Vec<String>, AppError> {
    let mut conn = client.get_multiplexed_async_connection().await?;
    let members: Vec<String> = conn.smembers(key).await?;

    Ok(members)
}

async fn backup_hashsets_to_redis(client: &Client, key: &str, data: HashMap<String, i32>) -> Result<(), AppError> {
    let mut conn = client.get_multiplexed_async_connection().await?;

    for (field, count) in data {
        conn.hset(key, field, count).await?;
    }

    Ok(())
}

async fn backup_sets_to_redis(client: &Client, key: &str, members: Vec<String>) -> Result<(), AppError> {
    let mut conn = client.get_multiplexed_async_connection().await?;

    for member in members {
        conn.sadd(key, member).await?;
    }

    Ok(())
}

async fn save_data(source_client: Client, backup_client: Client) -> Result<(), AppError> {
    let likes_hashset_key = "user-interaction:posts:likes";

    let likes_hashset_data = fetch_all_hashsets(&source_client, likes_hashset_key).await?;
    backup_hashsets_to_redis(&backup_client, likes_hashset_key, likes_hashset_data.clone()).await?;

    for (post_id, _) in likes_hashset_data.iter() {
        let likes_set_key = format!("user-interaction:posts:{}:likes", post_id);
        let likes_members = fetch_all_sets(&source_client, &likes_set_key).await?;
        backup_sets_to_redis(&backup_client, &likes_set_key, likes_members).await?;
    }

    let views_hashset_key = "user-interaction:posts:views";
    let views_hashset_data = fetch_all_hashsets(&source_client, views_hashset_key).await?;
    backup_hashsets_to_redis(&backup_client, views_hashset_key, views_hashset_data.clone()).await?;

    for (post_id, _) in views_hashset_data.iter() {
        let views_set_key = format!("user-interaction:posts:{}:views", post_id);
        let views_members = fetch_all_sets(&source_client, &views_set_key).await?;
        backup_sets_to_redis(&backup_client, &views_set_key, views_members).await?;
    }

    Ok(())
}