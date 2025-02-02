use crate::errors;
use crate::models::reply::Reply;
use crate::models::Finalizer;
use crate::repositories::reply_repo::{PostgresReplyRepository, ReplyRepository};
use async_trait::async_trait;
use opentelemetry::metrics::{Counter, Histogram};
use opentelemetry::trace::SpanKind;
use opentelemetry::{global, KeyValue};
use std::collections::HashMap;
use sqlx::PgPool;
use tokio::time::Instant;
use tracing::{field, info_span};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use ulid::Ulid;

pub struct DecoratedReplyRepository<R: ReplyRepository> {
    reply_repo: R,
}

impl DecoratedReplyRepository<PostgresReplyRepository> {
    pub fn new(db: PgPool) -> Self {
        Self {
            reply_repo: PostgresReplyRepository::new(db),
        }
    }
}

impl<U: ReplyRepository + 'static> DecoratedReplyRepository<U> {
    pub fn observable(self) -> DecoratedReplyRepository<ObservableReplyRepository<U>> {
        DecoratedReplyRepository {
            reply_repo: ObservableReplyRepository::new(self.reply_repo),
        }
    }

    pub fn build(self) -> U {
        self.reply_repo
    }
}



#[derive(Clone)]
pub struct ObservableReplyRepository<R: ReplyRepository> {
    inner: R,
    request_count: Counter<u64>,
    request_latency: Histogram<f64>,
}

impl<R: ReplyRepository> ObservableReplyRepository<R> {
    pub fn new(inner: R) -> Self {
        let boundaries = vec![
            0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
        ];

        let meter_provider = global::meter("user-interaction");
        let request_count = meter_provider
            .u64_counter("reply_repo_requests_total")
            .with_description("Total requests to ReplyRepository")
            .with_unit("ReplyRepository")
            .build();

        let request_latency = meter_provider
            .f64_histogram("reply_repo_request_duration_seconds")
            .with_description("Latency of ReplyRepository methods")
            .with_boundaries(boundaries)
            .with_unit("ReplyRepository")
            .build();

        Self {
            inner,
            request_count,
            request_latency,
        }
    }

    async fn track_method<T, F, E: errors::ProblemResponse + ToString>(
        &self,
        method_name: &str,
        query_summary: &str,
        operation_name: &str,
        target: &str,
        operation: F,
    ) -> Result<T, E>
    where
        F: std::future::Future<Output = Result<T, E>>,
    {
        let start_time = Instant::now();
        let span = info_span!(
            "",
            "otel.name" = query_summary,
            "otel.kind" = ?SpanKind::Client,
            "db.system.name" = "postgresql",
            "db.operation.name" = operation_name,
            "db.target" = target,
            "method.name" = method_name,
            "error.message" = field::Empty,
            "error.type" = field::Empty,
        );

        let result = operation.await;
        let status = if result.is_ok() { "success" } else { "error" };
        let mut attributes = vec![
            KeyValue::new("method", method_name.to_string()),
            KeyValue::new("operation", operation_name.to_string()),
        ];

        self.request_latency.record(
            start_time.elapsed().as_secs_f64(),
            &attributes,
        );
        
        attributes.push(KeyValue::new("status", status));
        self.request_count.add(
            1,
            &attributes,
        );

        if let Err(ref err) = result {
            span.record("error.type", "database_error")
                .set_status(opentelemetry::trace::Status::error(err.to_string()));

            return result;
        }

        span.set_status(opentelemetry::trace::Status::Ok);
        result
    }
}

#[async_trait]
impl<R: ReplyRepository + 'static> Finalizer for ObservableReplyRepository<R> {
    async fn finalize(&self) -> Result<(), errors::AppError> {
        self.inner.finalize().await
    }
}

#[async_trait]
impl<R: ReplyRepository + 'static> ReplyRepository for ObservableReplyRepository<R> {
    async fn get_all_from_post(&self, post_id: &Ulid) -> Result<Vec<Reply>, errors::DatabaseError> {
        self.track_method(
            "get_all_from_post",
            "SELECT replies",
            "SELECT",
            "replies",
            self.inner.get_all_from_post(post_id),
        )
        .await
    }

    async fn get_all_from_posts(
        &self,
        post_ids: &[Ulid],
    ) -> Result<HashMap<Ulid, Vec<Reply>>, errors::DatabaseError> {
        self.track_method(
            "get_all_from_posts",
            "SELECT replies",
            "SELECT",
            "replies",
            self.inner.get_all_from_posts(post_ids),
        )
        .await
    }

    async fn get_with_nested_by_path_prefix(
        &self,
        prefix: &str,
    ) -> Result<Vec<Reply>, errors::DatabaseError> {
        self.track_method(
            "get_with_nested_by_path_prefix",
            "SELECT replies",
            "SELECT",
            "replies",
            self.inner.get_with_nested_by_path_prefix(prefix),
        )
        .await
    }

    async fn get_with_nested(&self, id: &Ulid) -> Result<Vec<Reply>, errors::DatabaseError> {
        self.track_method(
            "get_with_nested",
            "SELECT replies",
            "SELECT",
            "replies",
            self.inner.get_with_nested(id),
        )
        .await
    }

    async fn create(
        &self,
        post_id: Ulid,
        parent_id: Ulid,
        content: &str,
        user_id: Ulid,
    ) -> Result<Reply, errors::DatabaseError> {
        self.track_method(
            "create",
            "SELECT INSERT replies",
            "SELECT INSERT",
            "replies",
            self.inner.create(post_id, parent_id, content, user_id),
        )
        .await
    }

    async fn update(&self, id: &Ulid, content: &str) -> Result<Reply, errors::DatabaseError> {
        self.track_method(
            "update",
            "UPDATE replies",
            "UPDATE",
            "replies",
            self.inner.update(id, content),
        )
        .await
    }

    async fn delete(&self, id: &Ulid) -> Result<(), errors::DatabaseError> {
        self.track_method(
            "delete",
            "DELETE replies",
            "DELETE",
            "replies",
            self.inner.delete(id),
        )
        .await
    }

    async fn delete_all_by_user_id(&self, id: &Ulid) -> Result<Vec<String>, errors::DatabaseError> {
        self.track_method(
            "delete_all_by_user_id",
            "DELETE replies by user_id",
            "DELETE",
            "replies",
            self.inner.delete_all_by_user_id(id),
        )
        .await
    }
}
