use crate::services::s3::S3Service;
use crate::services::user_profile::{UserProfileService};
use crate::settings;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState<S: S3Service> {
    pub db: mongodb::Database,
    pub redis: redis::Client,
    pub s3file_service: Arc<S>,
    pub user_profile_service: UserProfileService,
    pub amq: lapin::Channel,
    pub config: settings::AppConfig,
}
