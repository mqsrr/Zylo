use crate::errors;
use crate::models::reply::PostInteractionResponse;
use crate::repositories::interaction_repo::InteractionRepository;
use crate::services::reply_service::ReplyService;
use crate::utils::helpers::PostInteractionResponseBuilder;
use async_trait::async_trait;
use std::sync::Arc;
use ulid::Ulid;

#[async_trait]
pub trait PostInteractionsService: Send + Sync {
    async fn get_post_interactions(
        &self,
        post_id: Ulid,
        user_id: Option<Ulid>,
    ) -> Result<PostInteractionResponse, errors::AppError>;
    async fn get_posts_interactions(
        &self,
        posts_ids: &[Ulid],
        user_id: Option<Ulid>,
    ) -> Result<Vec<PostInteractionResponse>, errors::AppError>;
}

pub struct PostInteractionsServiceImpl<RS, I>
where
    RS: ReplyService + 'static,
    I: InteractionRepository + 'static,
{
    reply_service: Arc<RS>,
    interaction_repo: Arc<I>,
}

impl<RS, I> PostInteractionsServiceImpl<RS, I>
where
RS: ReplyService + 'static,
I: InteractionRepository + 'static,
{
    pub fn new(reply_service: Arc<RS>, interaction_repo: Arc<I>) -> Self {
        Self { reply_service, interaction_repo }
    }
}

#[async_trait]
impl<RS, I> PostInteractionsService for PostInteractionsServiceImpl<RS, I>
where
    RS: ReplyService + 'static,
    I: InteractionRepository + 'static,
{
    async fn get_post_interactions(
        &self,
        post_id: Ulid,
        user_id: Option<Ulid>,
    ) -> Result<PostInteractionResponse, errors::AppError> {
        let replies = self
            .reply_service
            .get_replies_from_post(&post_id, user_id)
            .await?;

        let post_id_string = post_id.to_string();
        let user_interacted_on_post = match user_id {
            Some(uid) => {
                let is_liked = self
                    .interaction_repo
                    .is_liked(&post_id_string, &uid)
                    .await?;

                Ok::<Option<bool>, errors::RedisError>(Some(is_liked))
            }
            None => Ok(None),
        }?;

        let (post_likes, post_views) = tokio::try_join!(
            self.interaction_repo.get_likes(&post_id_string),
            self.interaction_repo.get_views(&post_id_string)
        )?;

        let response = PostInteractionResponseBuilder::new()
            .post_id(post_id)
            .replies(replies)
            .likes(post_likes)
            .views(post_views)
            .user_interacted(user_interacted_on_post)
            .build();

        Ok(response)
    }

    async fn get_posts_interactions(
        &self,
        posts_ids: &[Ulid],
        user_id: Option<Ulid>,
    ) -> Result<Vec<PostInteractionResponse>, errors::AppError> {
        let replies_map = self
            .reply_service
            .get_replies_from_posts(posts_ids, user_id)
            .await?;

        let posts_ids_string = posts_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>();

        let (posts_likes, posts_views) = tokio::try_join!(
            self.interaction_repo.get_many_likes(&posts_ids_string),
            self.interaction_repo.get_many_views(&posts_ids_string)
        )?;

        let is_liked_map = match user_id {
            Some(uid) => Some(self.interaction_repo.is_many_liked(&posts_ids_string, &uid).await?),
            None => None,
        };

        let mut responses = Vec::new();
        for post_id in posts_ids {
            let post_id_string = post_id.to_string();
            let replies = replies_map.get(post_id).cloned();
            let user_interacted = is_liked_map
                .as_ref()
                .and_then(|map| map.get(&post_id_string).copied());
            
            let likes = posts_likes.get(&post_id_string).copied().unwrap_or_default();
            let views = posts_views.get(&post_id_string).copied().unwrap_or_default();

            let mut builder = PostInteractionResponseBuilder::new()
                .post_id(*post_id)
                .likes(likes)
                .views(views)
                .user_interacted(user_interacted);

            if let Some(replies) = replies {
                builder = builder.replies(replies);
            }

            responses.push(builder.build());
        } 
       

        Ok(responses)
    }
}
