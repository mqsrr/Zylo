use crate::errors;
use crate::repositories::interaction_repo::InteractionRepository;
use crate::repositories::reply_repo::ReplyRepository;
use crate::settings::AppConfig;
use std::sync::Arc;
use crate::services::amq_client::AmqClient;
use crate::services::cache_service::CacheService;

#[derive(Debug)]
pub struct AppState<R, I, A, C>
where
    R: ReplyRepository + 'static,
    I: InteractionRepository + 'static,
    A: AmqClient + 'static,
    C: CacheService + 'static,
{
    pub reply_repo: Arc<R>,
    pub interaction_repo: Arc<I>,
    pub amq_client: Arc<A>,
    pub cache_service: Arc<C>,
    pub config: AppConfig,
}

impl<R, I, A, C> Clone for AppState<R, I, A, C>
where
    R: ReplyRepository + 'static,
    I: InteractionRepository + 'static,
    A: AmqClient + 'static,
    C: CacheService + 'static,
{
    fn clone(&self) -> Self {
        AppState {
            reply_repo: self.reply_repo.clone(),
            interaction_repo: self.interaction_repo.clone(),
            amq_client: self.amq_client.clone(),
            cache_service: self.cache_service.clone(),
            config: self.config.clone(),
        }
    }
}
impl<R, I, A, C> AppState<R, I, A, C>
where
    R: ReplyRepository + 'static,
    I: InteractionRepository + 'static,
    A: AmqClient + 'static,
    C: CacheService + 'static,
{
    pub fn new(
        reply_repo: Arc<R>,
        interaction_repo: Arc<I>,
        amq_client: Arc<A>,
        cache_service: Arc<C>,
        config: AppConfig,
    ) -> Self {
        AppState {
            reply_repo,
            interaction_repo,
            amq_client,
            cache_service,
            config,
        }
    }

    pub async fn close(&self) -> Result<(), errors::AppError> {
        self.reply_repo.finalize().await?;
        self.amq_client.finalize().await?;

        Ok(())
    }
}
