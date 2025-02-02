use crate::errors;
use crate::models::reply::ReplyResponse;
use crate::models::Finalizer;
use crate::repositories::interaction_repo::InteractionRepository;
use crate::repositories::reply_repo::ReplyRepository;
use crate::utils::helpers::map_nested;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use ulid::Ulid;

#[async_trait]
pub trait ReplyService: Send + Sync + Finalizer {
    async fn get_replies_from_post(
        &self,
        post_id: &Ulid,
        user_id: Option<Ulid>,
    ) -> Result<Vec<ReplyResponse>, errors::AppError>;
    async fn get_replies_from_posts(
        &self,
        posts_ids: &[Ulid],
        user_id: Option<Ulid>,
    ) -> Result<HashMap<Ulid, Vec<ReplyResponse>>, errors::AppError>;
    async fn get(
        &self,
        reply_id: &Ulid,
        user_id: Option<Ulid>,
    ) -> Result<ReplyResponse, errors::AppError>;
    async fn create(
        &self,
        post_id: Ulid,
        parent_id: Ulid,
        content: &str,
        user_id: Ulid,
    ) -> Result<ReplyResponse, errors::AppError>;
    async fn update(
        &self,
        reply_id: &Ulid,
        content: &str,
    ) -> Result<ReplyResponse, errors::AppError>;
    async fn delete(&self, reply_id: &Ulid) -> Result<(), errors::AppError>;
}

pub struct ReplyServiceImpl<R: ReplyRepository + 'static, I: InteractionRepository + 'static> {
    reply_repo: Arc<R>,
    interaction_repo: Arc<I>,
}

impl<R: ReplyRepository + 'static, I: InteractionRepository + 'static> ReplyServiceImpl<R, I> {
    pub fn new(reply_repo: Arc<R>, interaction_repo: Arc<I>) -> Self {
        Self {
            reply_repo,
            interaction_repo,
        }
    }

    async fn populate_interactions(
        &self,
        replies: &mut [ReplyResponse],
        user_id: Option<Ulid>,
    ) -> Result<(), errors::AppError> {
        let reply_ids: Vec<String> = replies.iter().map(|r| r.id.to_string()).collect();

        let (likes, views) = tokio::try_join!(
            self.interaction_repo.get_many_likes(&reply_ids),
            self.interaction_repo.get_many_views(&reply_ids)
        )?;

        let user_interactions = if let Some(uid) = user_id {
            self.interaction_repo
                .is_many_liked(&reply_ids, &uid)
                .await?
        } else {
            HashMap::default()
        };

        for reply in replies.iter_mut() {
            let reply_id = reply.id.to_string();

            reply.likes = likes.get(&reply_id).copied().unwrap_or(0);
            reply.views = views.get(&reply_id).copied().unwrap_or(0);

            if !user_interactions.is_empty() {
                reply.user_interacted = user_interactions.get(&reply.id.to_string()).copied();
            }
        }

        Ok(())
    }
}

#[async_trait]
impl<R: ReplyRepository + 'static, I: InteractionRepository + 'static> Finalizer
    for ReplyServiceImpl<R, I>
{
    async fn finalize(&self) -> Result<(), errors::AppError> {
        self.reply_repo.finalize().await
    }
}

#[async_trait]
impl<R: ReplyRepository + 'static, I: InteractionRepository + 'static> ReplyService
    for ReplyServiceImpl<R, I>
{
    async fn get_replies_from_post(
        &self,
        post_id: &Ulid,
        user_id: Option<Ulid>,
    ) -> Result<Vec<ReplyResponse>, errors::AppError> {
        let mut replies: Vec<ReplyResponse> = self
            .reply_repo
            .get_all_from_post(post_id)
            .await?
            .into_iter()
            .map(ReplyResponse::from)
            .collect();

        self.populate_interactions(&mut replies, user_id).await?;
        Ok(map_nested(replies)
               .into_values()
               .flatten()
               .collect::<Vec<_>>())
    }

    async fn get_replies_from_posts(
        &self,
        posts_ids: &[Ulid],
        user_id: Option<Ulid>,
    ) -> Result<HashMap<Ulid, Vec<ReplyResponse>>, errors::AppError> {
        let grouped_replies_map = self.reply_repo.get_all_from_posts(posts_ids).await?;
        let mut replies_with_post: Vec<ReplyResponse> = grouped_replies_map
            .iter()
            .flat_map(|(_, replies)| {
                replies
                    .clone()
                    .into_iter()
                    .map(ReplyResponse::from)
                    .collect::<Vec<_>>()
            })
            .collect();

        self.populate_interactions(&mut replies_with_post, user_id)
            .await?;
        let post_hierarchies = map_nested(replies_with_post);

        Ok(post_hierarchies)
    }

    async fn get(
        &self,
        reply_id: &Ulid,
        user_id: Option<Ulid>,
    ) -> Result<ReplyResponse, errors::AppError> {
        let mut replies: Vec<ReplyResponse> = self
            .reply_repo
            .get_with_nested(reply_id)
            .await?
            .into_iter()
            .map(ReplyResponse::from)
            .collect();

        self.populate_interactions(&mut replies, user_id).await?;
        Ok(map_nested(replies)
            .into_values()
            .flatten()
            .collect::<Vec<_>>()
            .pop()
            .ok_or(errors::AppError::NotFound(String::from("Reply could not be found")))?)
    }

    async fn create(
        &self,
        post_id: Ulid,
        parent_id: Ulid,
        content: &str,
        user_id: Ulid,
    ) -> Result<ReplyResponse, errors::AppError> {
        let reply = self
            .reply_repo
            .create(post_id, parent_id, content, user_id)
            .await?;

        Ok(reply.into())
    }

    async fn update(
        &self,
        reply_id: &Ulid,
        content: &str,
    ) -> Result<ReplyResponse, errors::AppError> {
        let updated = self.reply_repo.update(reply_id, content).await?;
        Ok(updated.into())
    }

    async fn delete(&self, reply_id: &Ulid) -> Result<(), errors::AppError> {
        self.reply_repo.delete(reply_id).await?;

        self.interaction_repo
            .delete_interactions(&reply_id.to_string())
            .await
            .map_err(Into::into)
    }
}
