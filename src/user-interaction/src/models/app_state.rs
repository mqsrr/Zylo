use redis::Client;
use sqlx::PgPool;
use crate::services::user_profile::UserProfileService;
use crate::setting::AppConfig;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: Client,
    pub user_profile_service: UserProfileService,
    pub amq: lapin::Channel,
    pub config: AppConfig
}