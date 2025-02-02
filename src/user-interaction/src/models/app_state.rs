use crate::errors;
use crate::services::amq_client::AmqClient;
use crate::services::reply_service::ReplyService;
use crate::settings::AppConfig;
use std::sync::Arc;
use crate::repositories::interaction_repo::InteractionRepository;
use crate::services::post_interactions_service::PostInteractionsService;

#[derive(Debug)]
pub struct AppState<A,I, RS, PS>
where
    A: AmqClient + 'static,
    I: InteractionRepository + 'static,
    RS: ReplyService + 'static,
    PS: PostInteractionsService + 'static,
{
    pub amq_client: Arc<A>,
    pub interaction_repo: Arc<I>,
    pub reply_service: Arc<RS>,
    pub post_interactions_service: Arc<PS>,
    pub config: AppConfig,
}

impl<A, I, RS, PS> Clone for AppState<A, I, RS, PS>
where
    A: AmqClient + 'static,
    I: InteractionRepository + 'static,
    RS: ReplyService + 'static,
    PS: PostInteractionsService + 'static,
{
    fn clone(&self) -> Self {
        AppState {
            amq_client: self.amq_client.clone(),
            interaction_repo: self.interaction_repo.clone(),
            reply_service: self.reply_service.clone(),
            post_interactions_service: self.post_interactions_service.clone(),
            config: self.config.clone(),
        }
    }
}
impl<A,I, RS, PS> AppState<A, I, RS, PS>
where
    A: AmqClient + 'static,
    I: InteractionRepository + 'static,
    RS: ReplyService + 'static,
    PS: PostInteractionsService + 'static,
{
    pub fn new(
        amq_client: Arc<A>,
        interaction_repo: Arc<I>,
        reply_service: Arc<RS>,
        post_interactions_service: Arc<PS>,
        config: AppConfig,
    ) -> Self {
        AppState {
            amq_client,
            interaction_repo,
            reply_service,
            post_interactions_service,
            config,
        }
    }

    pub async fn close(&self) -> Result<(), errors::AppError> {
        self.reply_service.finalize().await?;
        self.amq_client.finalize().await?;

        Ok(())
    }
}
