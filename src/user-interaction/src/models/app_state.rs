use sqlx::PgPool;
use crate::services::{amq, database, redis};
use crate::services::user_profile::user_profile::user_profile_service_client::UserProfileServiceClient;
use crate::services::user_profile::UserProfileService;
use crate::setting::AppConfig;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: ::redis::Client,
    pub user_profile_service: UserProfileService,
    pub amq: lapin::Channel,
    pub config: AppConfig,
}

impl AppState {
    pub async fn new(config: &AppConfig) -> Self {
        let db = database::init_db(&config.database).await;
        let redis = redis::init_client(&config.redis).await;
        let amq = amq::open_amq_connection(&config.amq).await;
        let user_profile_service = UserProfileService::new(redis.clone(), UserProfileServiceClient::connect(config.grpc_server.uri.clone()).await.unwrap());

        amq::consume_user_created(&amq, db.clone()).await.unwrap();
        amq::consume_post_deleted(&amq, db.clone(), redis.clone()).await.unwrap();

        amq::consume_user_updated(&amq, db.clone(), redis.clone()).await.unwrap();
        amq::consume_user_deleted(&amq, db.clone(), redis.clone()).await.unwrap();

        AppState {
            db,
            redis,
            user_profile_service,
            amq,
            config: config.clone(),
        }
    }

    pub async fn close(&self) {
        self.db.close().await;
        self.amq.close(200, "Application is shutting down").await.unwrap();
    }
}