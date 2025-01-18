use crate::errors;
use crate::errors::redis_op_error;
use crate::settings;
use redis::{AsyncCommands, Client};
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::log::{error, info};

pub async fn start_worker(redis_config: settings::Redis) -> Result<(), errors::RedisError> {
    let sched = JobScheduler::new().await.map_err(|e| {
        errors::RedisError::BackgroundWorkerStartupError(format!(
            "Failed to create scheduler: {:?}",
            e
        ))
    })?;

    let job = Job::new_cron_job_async("0 */10 * * * *", move |_uuid, _l| {
        Box::pin({
            {
                let value = redis_config.clone();
                async move {
                    info!("Starting backup of Redis data");
                    if let Err(e) = run_backup(value).await {
                        error!("Backup failed: {}", e);
                    } else {
                        info!("Backup completed successfully.");
                    }
                }
            }
        })
    })
    .map_err(|e| {
        errors::RedisError::BackgroundWorkerStartupError(format!(
            "Failed to create cron job: {:?}",
            e
        ))
    })?;

    sched.add(job).await.map_err(|e| {
        errors::RedisError::BackgroundWorkerStartupError(format!("Failed to add job: {:?}", e))
    })?;

    sched.shutdown_on_ctrl_c();
    sched.start().await.map_err(|e| {
        errors::RedisError::BackgroundWorkerStartupError(format!("Scheduler start failed: {:?}", e))
    })?;

    Ok(())
}

async fn run_backup(redis_config: settings::Redis) -> Result<(), errors::RedisError> {
    let source_client = create_redis_client(&redis_config.uri).await?;
    let backup_client = create_redis_client(&redis_config.backup_uri).await?;

    save_data(source_client, backup_client).await
}

async fn create_redis_client(uri: &str) -> Result<Client, errors::RedisError> {
    Client::open(uri.to_string()).map_err(|_| errors::RedisError::ConnectionError)
}

async fn fetch_all_hashset_values(
    client: &Client,
    key: &str,
) -> Result<Vec<(String, i32)>, errors::RedisError> {
    let mut conn = client
        .get_multiplexed_async_connection()
        .await
        .map_err(|_| errors::RedisError::ConnectionError)?;

    let hashset_data: Vec<(String, i32)> = conn
        .hgetall(key)
        .await
        .map_err(|e| redis_op_error("HGETALL", key, e))?;

    Ok(hashset_data)
}

async fn fetch_all_set_values(
    client: &Client,
    key: &str,
) -> Result<Vec<String>, errors::RedisError> {
    let mut conn = client
        .get_multiplexed_async_connection()
        .await
        .map_err(|_| errors::RedisError::ConnectionError)?;

    let members: Vec<String> = conn
        .smembers(key)
        .await
        .map_err(|e| redis_op_error("HGETALL", key, e))?;

    Ok(members)
}

async fn backup_hashsets_to_redis(
    client: &Client,
    key: &str,
    data: &[(String, i32)],
) -> Result<(), errors::RedisError> {
    if data.is_empty() {
        return Ok(());
    }

    let mut conn = client
        .get_multiplexed_async_connection()
        .await
        .map_err(|_| errors::RedisError::ConnectionError)?;

    conn.hset_multiple(key, data)
        .await
        .map_err(|e| redis_op_error("BACKUP_HMSET", key, e))?;

    Ok(())
}

async fn backup_sets_to_redis(
    client: &Client,
    key: &str,
    members: Vec<String>,
) -> Result<(), errors::RedisError> {
    if members.is_empty() {
        return Ok(());
    }

    let mut conn = client
        .get_multiplexed_async_connection()
        .await
        .map_err(|_| errors::RedisError::ConnectionError)?;

    conn.sadd(key, members)
        .await
        .map_err(|e| redis_op_error("BACKUP_SADD", key, e))?;

    Ok(())
}

async fn save_data(source_client: Client, backup_client: Client) -> Result<(), errors::RedisError> {
    let created_users_key = "user-interaction:created-users";
    let created_users = fetch_all_hashset_values(&source_client, created_users_key).await?;
    backup_hashsets_to_redis(&backup_client, created_users_key, &created_users).await?;

    let created_posts_key = "user-interaction:created-posts";
    let created_posts = fetch_all_hashset_values(&source_client, created_posts_key).await?;
    backup_hashsets_to_redis(&backup_client, created_posts_key, &created_posts).await?;

    let likes_hashset_key = "user-interaction:posts:likes";
    let likes_hashset_data = fetch_all_hashset_values(&source_client, likes_hashset_key).await?;
    backup_hashsets_to_redis(&backup_client, likes_hashset_key, &likes_hashset_data).await?;

    for (post_id, _) in created_posts.iter() {
        let likes_set_key = format!("user-interaction:posts:{}:likes", post_id);
        let likes_members = fetch_all_set_values(&source_client, &likes_set_key).await?;
        backup_sets_to_redis(&backup_client, &likes_set_key, likes_members).await?;
    }

    let views_hashset_key = "user-interaction:posts:views";
    let views_hashset_data = fetch_all_hashset_values(&source_client, views_hashset_key).await?;
    backup_hashsets_to_redis(&backup_client, views_hashset_key, &views_hashset_data).await?;

    for (post_id, _) in created_posts.iter() {
        let views_set_key = format!("user-interaction:posts:{}:views", post_id);
        let views_members = fetch_all_set_values(&source_client, &views_set_key).await?;
        backup_sets_to_redis(&backup_client, &views_set_key, views_members).await?;
    }

    Ok(())
}
