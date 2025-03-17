use std::ops::Not;
use std::collections::{HashMap};
use std::sync::Arc;
use crate::services::aggregator::{BatchOfPostInteractionsResponse, FileMetadataResponse, GrpcUserPreview, PaginatedPostsResponse, PostInteractionsResponse, PostResponse, ReplyResponse, UserImage};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedResponse<T> {
    data: Vec<T>,
    per_page: u32,
    next: String,
    has_next_page: bool,
    #[serde(skip_serializing_if = "<&bool>::not")]
    is_stale: bool
}

impl PaginatedResponse<Post> {
    pub fn new(data: Vec<Post>, per_page: u32, next: String, has_next_page: bool, is_stale: bool) -> Self {
        Self {
            data,
            per_page,
            next,
            has_next_page,
            is_stale
        }
    }
    
    pub fn from(
        paginated_posts: PaginatedPostsResponse,
        batch_posts_interactions: BatchOfPostInteractionsResponse,
        users_map: &HashMap<String, Arc<UserSummary>>,
        is_stale: bool
    ) -> Self {
        let per_page = paginated_posts.per_page;
        let next = paginated_posts.next_cursor.clone();
        let has_next_page = paginated_posts.has_next_page;
        let data = Post::map_posts(paginated_posts.posts, batch_posts_interactions, users_map);
        
        Self {
            data,
            per_page,
            next,
            has_next_page,
            is_stale
        }
    }
}

#[derive(Serialize,Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct UserSummary {
    id: String,
    name: String,
    profile_image: FileMetadata,
}

impl From<GrpcUserPreview> for UserSummary {
    fn from(value: GrpcUserPreview) -> Self {
        Self {
            id: value.id,
            name: value.name,
            profile_image: FileMetadata::from(value.profile_image.unwrap_or_default()),
        }
    }
}

#[derive(Serialize,Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Post {
    id: String,
    user: Arc<UserSummary>,
    content: String,
    files: Vec<FileMetadata>,
    replies: Vec<Reply>,
    likes: u64,
    views: u64,
    user_interacted: bool,
    created_at: String,
    updated_at: String,
}

impl Post {
    pub fn map_posts(
        posts: Vec<PostResponse>,
        batch_posts_interactions: BatchOfPostInteractionsResponse,
        users_map: &HashMap<String, Arc<UserSummary>>,
    ) -> Vec<Post> {
        let mut interactions_map: HashMap<String, _> = batch_posts_interactions
            .posts_interactions
            .into_iter()
            .map(|pi| (pi.post_id.clone(), pi))
            .collect();

            posts
            .into_iter()
            .map(|post| {
                let post_interaction = interactions_map
                    .remove(&post.id)
                    .unwrap_or_default();
                Post::from(post, post_interaction, &users_map)
            })
            .collect()
    }
    
    pub fn from(
        post_response: PostResponse,
        post_interaction: PostInteractionsResponse,
        users_map: &HashMap<String, Arc<UserSummary>>,
    ) -> Self {
        let replies = post_interaction
            .replies
            .into_iter()
            .map(|reply| Reply::from(reply, users_map))
            .collect();
        
        Self {
            id: post_response.id,
            user: users_map.get(&post_response.user_id).unwrap_or(&Arc::<UserSummary>::default()).clone(),
            content: post_response.text,
            files: post_response
                .files_metadata
                .into_iter()
                .map(FileMetadata::from)
                .collect(),
            replies,
            likes: post_interaction.likes,
            views: post_interaction.views,
            user_interacted: post_interaction.user_interacted,
            created_at: post_response.created_at,
            updated_at: post_response.updated_at,
        }
    }
}

#[derive(Serialize,Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct FileMetadata {
    url: String,
    file_name: String,
    content_type: String,
}

impl From<FileMetadataResponse> for FileMetadata {
    fn from(value: FileMetadataResponse) -> Self {
        Self {
            url: value.url,
            file_name: value.file_name,
            content_type: value.content_type,
        }
    }
}
impl From<UserImage> for FileMetadata {
    fn from(value: UserImage) -> Self {
        Self {
            url: value.url,
            file_name: value.file_name,
            content_type: value.content_type,
        }
    }
}
#[derive(Serialize,Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Reply {
    id: String,
    user: Arc<UserSummary>,
    content: String,
    reply_to_id: String,
    views: u64,
    likes: u64,
    nested_replies: Vec<Reply>,
    user_interacted: bool,
    created_at: String,
}

impl Reply {
    pub fn from(value: ReplyResponse, users_map: &HashMap<String, Arc<UserSummary>>) -> Self {
        Self {
            id: value.id,
            user: users_map.get(&value.user_id).unwrap_or(&Arc::<UserSummary>::default()).clone(),
            content: value.content,
            reply_to_id: value.reply_to_id,
            views: value.views,
            likes: value.likes,
            nested_replies: value
                .nested_replies
                .into_iter()
                .map(|reply| Reply::from(reply, users_map))
                .collect(),
            user_interacted: value.user_interacted,
            created_at: DateTime::<Utc>::from_timestamp_nanos(value.created_at).to_rfc3339(),
        }
    }
}
