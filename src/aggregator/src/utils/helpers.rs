use std::collections::{HashMap, HashSet};
use std::fs;
use std::sync::Arc;
use tonic::IntoRequest;
use tonic::transport::Channel;
use crate::errors;
use crate::models::post::UserSummary;
use crate::services::aggregator::{BatchOfPostInteractionsResponse, BatchUsersSummaryResponse, GetBatchOfPostInteractionsRequest, GetBatchUsersByIdsRequest, PostInteractionsResponse, PostResponse, ReplyResponse};
use crate::services::aggregator::reply_service_client::ReplyServiceClient;
use crate::services::aggregator::user_profile_service_client::UserProfileServiceClient;
use crate::services::InjectTraceContext;

pub fn get_container_id() -> Option<String> {
    if let Ok(cgroup) = fs::read_to_string("/proc/self/cgroup") {
        for line in cgroup.lines() {
            if let Some(id) = line.split('/').last() {
                if id.len() >= 12 {
                    return Some(id.to_string());
                }
            }
        }
    }
    None
}
pub fn collect_user_ids_from_posts(
    posts: &[PostResponse],
    interactions: &BatchOfPostInteractionsResponse,
) -> HashSet<String> {
    let mut ids =   HashSet::new();
    for post in posts {
        ids.insert(post.user_id.clone());
    }

    for interactions in &interactions.posts_interactions {
        for reply in &interactions.replies {
            collect_user_ids_from_reply(reply, &mut ids)
        }
    }

    ids
}

pub fn collect_user_ids_from_post(
    post: &PostResponse,
    interactions: &PostInteractionsResponse,
) -> HashSet<String> {
    let mut ids = HashSet::new();
    ids.insert(post.user_id.clone());

    for reply in &interactions.replies {
        collect_user_ids_from_reply(reply, &mut ids)
    }

    ids
}

pub fn collect_user_ids_from_reply(reply: &ReplyResponse, ids: &mut HashSet<String>) {
    ids.insert(reply.user_id.clone());
    for nested in &reply.nested_replies {
        collect_user_ids_from_reply(nested, ids);
    }
}


pub async fn fetch_user_summaries(
    user_client: &mut UserProfileServiceClient<Channel>,
    user_ids: HashSet<String>,
) -> Result<HashMap<String, Arc<UserSummary>>, errors::GrpcError> {
    let request = GetBatchUsersByIdsRequest {
        user_ids: user_ids.into_iter().collect(),
    }
        .into_request()
        .inject_trace_context();

    let response = user_client
        .get_batch_users_summary_by_ids(request)
        .await?;
    Ok(build_user_summary_map(response.into_inner()))
}

pub fn build_user_summary_map(
    response: BatchUsersSummaryResponse,
) -> HashMap<String, Arc<UserSummary>> {
    response
        .users
        .into_iter()
        .map(|grpc_user| {
            let user_id = grpc_user.id.clone();
            (user_id, Arc::new(UserSummary::from(grpc_user)))
        })
        .collect()
}


pub async fn get_posts_interactions(
    reply_client: &mut ReplyServiceClient<Channel>,
    posts: &[PostResponse],
    interaction_user_id: String,
) -> Result<BatchOfPostInteractionsResponse, errors::GrpcError> {
    let request = GetBatchOfPostInteractionsRequest {
        posts_ids: posts.iter().map(|p| p.id.clone()).collect(),
        interaction_user_id,
    }
        .into_request()
        .inject_trace_context();

    Ok(reply_client
        .get_batch_of_post_interactions(request)
        .await?
        .into_inner())
}