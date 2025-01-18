use std::sync::Arc;
use ulid::Ulid;
use crate::errors;
use crate::repositories::interaction_repo::InteractionRepository;
use crate::utils::request::ReplyResponse;

pub fn build_cache_key(user_id: Option<String>, post_id: &str) -> String {
    match user_id {
        Some(uid) => format!("*{}-{}*", uid, post_id),
        None => format!("*{}*", post_id),
    }
}

pub async fn populate_interactions_for_replies<I: InteractionRepository + 'static>(
    replies: &mut [ReplyResponse],
    interaction_repo: Arc<I>,
    user_id: Option<Ulid>,
) -> Result<(), errors::AppError> {
    let reply_ids: Vec<String> = replies.iter().map(|r| r.id.to_string()).collect();

    let likes = interaction_repo
        .get_interactions("user-interaction:posts:likes", &reply_ids)
        .await?;
    
    let views = interaction_repo
        .get_interactions("user-interaction:posts:views", &reply_ids)
        .await?;

    for reply in replies.iter_mut() {
        reply.likes = *likes.get(&reply.id).unwrap_or(&0);
        reply.views = *views.get(&reply.id).unwrap_or(&0);
    }

    if let Some(uid) = user_id {
        let user_interactions = interaction_repo
            .get_user_interactions(&uid.to_string(), reply_ids)
            .await?;

        for reply in replies.iter_mut() {
            reply.user_interacted = Some(
                *user_interactions
                    .get(&reply.id.to_string())
                    .unwrap_or(&false),
            );
        }
    }

    Ok(())
}