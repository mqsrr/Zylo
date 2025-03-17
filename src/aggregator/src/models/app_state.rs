use crate::services::post_service::PostsService;
use crate::services::user_service::UserService;
use crate::settings::AppConfig;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::services::feed_service::FeedService;

#[derive(Debug)]
pub struct AppState<P, U, F>
where
    P: PostsService + 'static,
    U: UserService + 'static,
    F: FeedService + 'static,
{
    pub posts_service: Arc<Mutex<P>>,
    pub users_service: Arc<Mutex<U>>,
    pub feed_service: Arc<Mutex<F>>,
    
    pub config: AppConfig,
}

impl<P, U, F> Clone for AppState<P, U, F>
where
    P: PostsService + 'static,
    U: UserService + 'static,
    F: FeedService + 'static,
{
    fn clone(&self) -> Self {
        AppState {
            posts_service: self.posts_service.clone(),
            users_service: self.users_service.clone(),
            feed_service: self.feed_service.clone(),
            config: self.config.clone(),
        }
    }
}
impl<P, U, F> AppState<P, U, F>
where
    P: PostsService + 'static,
    U: UserService + 'static,
    F: FeedService + 'static,
{
    pub fn new(
        posts_service: Arc<Mutex<P>>,
        users_service: Arc<Mutex<U>>,
        feed_service: Arc<Mutex<F>>,
        config: AppConfig,
    ) -> Self {
        AppState {
            posts_service,
            users_service,
            feed_service,
            config,
        }
    }
}