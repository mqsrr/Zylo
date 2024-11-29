use crate::services::s3::S3FileService;
use crate::services::user_profile::user_profile::user_profile_service_client::UserProfileServiceClient;
use crate::services::user_profile::UserProfileService;
use crate::services::{amq, database, redis, s3};
use crate::settings::{self, AppConfig};

#[derive(Clone)]
pub struct AppState {
    pub db: mongodb::Database,
    pub redis: ::redis::Client,
    pub s3file_service: S3FileService,
    pub user_profile_service: UserProfileService,
    pub amq: lapin::Channel,
    pub config: settings::AppConfig,
}

impl AppState {
    pub async fn new(config: &AppConfig) -> Self {
        let db = database::init_db(&config.database).await;
        let redis = redis::create_client(&config.redis).await;
        let amq = amq::open_amq_connection(&config.amq).await;

        let s3_client = s3::init_s3_client().await.unwrap();
        let s3_service = s3::S3FileService::new(s3_client, config.s3_config.clone());

        let user_profile = UserProfileService::new(
            redis.clone(),
            UserProfileServiceClient::connect(config.grpc_server.uri.clone())
                .await
                .unwrap(),
        );

        amq::consume_user_created(&amq, db.clone())
            .await
            .expect("Error configuring user created consumer");

        amq::consume_user_deleted(&amq, db.clone(), s3_service.clone())
            .await
            .expect("Error configuring user deleted consumer");

        amq::consume_user_updated(&amq, db.clone(), redis.clone())
            .await
            .expect("Error configuring user updated consumer");

        Self {
            db,
            redis,
            user_profile_service: user_profile,
            s3file_service: s3_service,
            amq,
            config: config.clone(),
        }
    }
}
