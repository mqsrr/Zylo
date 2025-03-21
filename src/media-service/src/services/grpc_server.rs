use crate::models::post::Post;
use crate::repositories::post_repo::PostRepository;
use crate::services::grpc_server::post_server::post_service_server::PostService;
use crate::services::grpc_server::post_server::{BatchPostsRequest, FileMetadataResponse, PaginatedPostsResponse, PostRequest, PostResponse, PostsRequest, PostsResponse};
use std::sync::Arc;
use opentelemetry::propagation::Extractor;
use tonic::{Request, Response, Status};
use ulid::Ulid;

pub mod post_server {
    tonic::include_proto!("post_server");
}

pub struct MetadataMap<'a>(pub &'a tonic::metadata::MetadataMap);

impl Extractor for MetadataMap<'_> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|metadata| metadata.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.0
            .keys()
            .map(|key| match key {
                tonic::metadata::KeyRef::Ascii(v) => v.as_str(),
                tonic::metadata::KeyRef::Binary(v) => v.as_str(),
            })
            .collect::<Vec<_>>()
    }
}

#[derive(Debug)]
pub struct GrpcPostServer<P>
where
    P: PostRepository + 'static,
{
    post_repo: Arc<P>,
}

impl<P> GrpcPostServer<P>
where
    P: PostRepository + 'static,
{
    pub fn new(post_repo: Arc<P>) -> Self {
        Self {
            post_repo,
        }
    }
}

#[tonic::async_trait]
impl<P> PostService for GrpcPostServer<P>
where
    P: PostRepository + 'static,
{
    async fn get_post_by_id(
        &self,
        request: Request<PostRequest>,
    ) -> Result<Response<PostResponse>, Status> {
        let post_id: Ulid = request
            .into_inner()
            .post_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid post id"))?;
        
        let post = self
            .post_repo
            .get(&post_id)
            .await?;
       
        Ok(Response::new(PostResponse::from(post)))
    }

    async fn get_paginated_posts(&self, request: Request<PostsRequest>) -> Result<Response<PaginatedPostsResponse>, Status> {
        let inner = request.into_inner();

        let user_id = inner
            .user_id
            .map(|user_id| Ulid::from_string(&user_id))
            .transpose()
            .map_err(|_| Status::invalid_argument("Invalid user id"))?;

        let per_page = inner.per_page as u32;
        let last_post_id = inner
            .last_post_id
            .map(|id| Ulid::from_string(&id))
            .transpose()
            .map_err(|err| Status::internal(err.to_string()))?;

        let posts = self
            .post_repo
            .get_paginated_posts(user_id, Some(per_page), last_post_id)
            .await?;

        Ok(Response::new(PaginatedPostsResponse {
            posts: posts.data.into_iter().map(PostResponse::from).collect(),
            has_next_page: posts.has_next_page,
            per_page,
            next_cursor: posts.next_cursor,
        }))
    }
    
    async fn get_batch_posts(
        &self,
        request: Request<BatchPostsRequest>,
    ) -> Result<Response<PostsResponse>, Status> {
        let post_ids = request
            .into_inner()
            .post_ids
            .into_iter()
            .map(|id| Ulid::from_string(&id))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| Status::invalid_argument("Invalid post id"))?;
        
        let posts = self
            .post_repo
            .get_batch_posts(post_ids)
            .await?;

        Ok(Response::new(PostsResponse {
            posts: posts.into_iter().map(PostResponse::from).collect(),
        }))
    }
}

impl From<Post> for PostResponse {
    fn from(value: Post) -> Self {
        Self {
            id: value.id.to_string(),
            user_id: value.user_id.to_string(),
            text: value.text,
            files_metadata: value
                .files_metadata
                .into_iter()
                .map(|file| FileMetadataResponse {
                    file_name: file.file_name,
                    content_type: file.content_type,
                    url: file.url.unwrap_or_default().url,
                })
                .collect(),
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
