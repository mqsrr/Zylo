use crate::errors;
use crate::repositories::post_repo::PostRepository;
use crate::repositories::user_repo::UsersRepository;
use crate::services::amq::AmqClient;
use crate::services::cache_service::CacheService;
use crate::settings::AppConfig;
use std::sync::Arc;

pub struct AppState<P, U, C, A>
where
    P: PostRepository + 'static,
    U: UsersRepository + 'static,
    C: CacheService + 'static,
    A: AmqClient + 'static,
{
    pub post_repo: Arc<P>,
    pub user_repo: Arc<U>,
    pub cache_service: Arc<C>,
    pub amq_client: Arc<A>,
    pub config: AppConfig,
}

impl<P, U, C, A> Clone for AppState<P, U, C, A>
where
    P: PostRepository + 'static,
    U: UsersRepository + 'static,
    C: CacheService + 'static,
    A: AmqClient + 'static,
{
    fn clone(&self) -> Self {
        Self {
            post_repo: self.post_repo.clone(),
            user_repo: self.user_repo.clone(),
            cache_service: self.cache_service.clone(),
            amq_client: self.amq_client.clone(),
            config: self.config.clone(),
        }
    }
}

impl<P, U, C, A> AppState<P, U, C, A>
where
    P: PostRepository + 'static,
    U: UsersRepository + 'static,
    C: CacheService + 'static,
    A: AmqClient + 'static,
{
    pub async fn new(
        post_repo: Arc<P>,
        user_repo: Arc<U>,
        cache_service: Arc<C>,
        amq_client: Arc<A>,
        config: AppConfig,
    ) -> Self {
        Self {
            post_repo,
            user_repo,
            cache_service,
            amq_client,
            config,
        }
    }

    pub async fn close(&self) -> Result<(), errors::AppError> {
        self.amq_client.finalize().await?;
        Ok(())
    }
}
