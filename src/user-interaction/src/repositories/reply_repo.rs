use crate::errors;
use crate::models::reply::Reply;
use crate::models::Finalizer;
use async_trait::async_trait;
use chrono::NaiveDateTime;
use sqlx::{PgPool};
use std::collections::HashMap;
use tracing::info;
use ulid::Ulid;

#[async_trait]
pub trait ReplyRepository: Send + Sync + Finalizer {
    async fn get_all_from_post(&self, post_id: &Ulid) -> Result<Vec<Reply>, errors::DatabaseError>;

    async fn get_all_from_posts(
        &self,
        post_ids: &[Ulid],
    ) -> Result<HashMap<Ulid, Vec<Reply>>, errors::DatabaseError>;

    async fn get_with_nested_by_path_prefix(
        &self,
        prefix: &str,
    ) -> Result<Vec<Reply>, errors::DatabaseError>;

    async fn get_with_nested(&self, id: &Ulid) -> Result<Vec<Reply>, errors::DatabaseError>;

    async fn create(
        &self,
        post_id: Ulid,
        parent_id: Ulid,
        content: &str,
        user_id: Ulid,
    ) -> Result<Reply, errors::DatabaseError>;

    async fn update(&self, id: &Ulid, content: &str) -> Result<Reply, errors::DatabaseError>;
    async fn delete(&self, id: &Ulid) -> Result<(), errors::DatabaseError>;
    async fn delete_all_by_user_id(&self, id: &Ulid) -> Result<Vec<String>, errors::DatabaseError>;
}

pub struct PostgresReplyRepository {
    pub pool: PgPool,
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct ReplyRow {
    pub id: Vec<u8>,
    pub post_id: Vec<u8>,
    pub user_id: Vec<u8>,
    pub reply_to_id: Vec<u8>,
    pub content: String,
    pub created_at: NaiveDateTime,
}

impl From<ReplyRow> for Reply {
    fn from(row: ReplyRow) -> Self {
        Reply {
            id: Ulid::from_bytes(row.id.try_into().unwrap()),
            post_id: Ulid::from_bytes(row.post_id.try_into().unwrap()),
            reply_to_id: Ulid::from_bytes(row.reply_to_id.try_into().unwrap()),
            user_id: Ulid::from_bytes(row.user_id.try_into().unwrap()),
            content: row.content,
            created_at: row.created_at,
        }
    }
}

impl PostgresReplyRepository {
    pub fn new(pool: PgPool) -> PostgresReplyRepository {
        Self { pool }
    }
}

#[async_trait]
impl ReplyRepository for PostgresReplyRepository {
    async fn get_all_from_post(&self, post_id: &Ulid) -> Result<Vec<Reply>, errors::DatabaseError> {
        let rows: Vec<ReplyRow> = sqlx::query_as(
            r#"
            SELECT
              id,
              post_id,
              reply_to_id,
              user_id,
              content,
              created_at,
              path
            FROM replies
            WHERE post_id = $1
            ORDER BY created_at ASC
            "#,
        )
        .bind(post_id.to_bytes())
        .fetch_all(&self.pool)
        .await?;

        let replies: Vec<Reply> = rows.into_iter().map(Reply::from).collect();
        Ok(replies)
    }

    async fn get_all_from_posts(
        &self,
        post_ids: &[Ulid],
    ) -> Result<HashMap<Ulid, Vec<Reply>>, errors::DatabaseError> {
        if post_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let post_id_bytes: Vec<Vec<u8>> =
            post_ids.iter().map(|id| id.to_bytes().to_vec()).collect();

        let rows: Vec<ReplyRow> = sqlx::query_as(
            r#"
            SELECT
              id,
              post_id,
              reply_to_id,
              user_id,
              content,
              created_at,
              path
            FROM replies
            WHERE post_id = ANY($1)
            ORDER BY created_at ASC
            "#,
        )
        .bind(&post_id_bytes)
        .fetch_all(&self.pool)
        .await?;

        let all_replies: Vec<Reply> = rows.into_iter().map(Reply::from).collect();
        let mut grouped_map: HashMap<Ulid, Vec<Reply>> = HashMap::new();
        for reply in all_replies {
            grouped_map.entry(reply.post_id).or_default().push(reply);
        }

        Ok(grouped_map)
    }

    async fn get_with_nested_by_path_prefix(
        &self,
        prefix: &str,
    ) -> Result<Vec<Reply>, errors::DatabaseError> {
        let rows: Vec<ReplyRow> = sqlx::query_as(
            r#"
            SELECT
              id,
              post_id,
              reply_to_id,
              user_id,
              content,
              created_at,
              path
            FROM replies
            WHERE path LIKE $1 || '%'
            ORDER BY created_at
            "#,
        )
        .bind(prefix)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(Reply::from).collect())
    }

    async fn get_with_nested(&self, id: &Ulid) -> Result<Vec<Reply>, errors::DatabaseError> {
        let rows: Vec<ReplyRow> = sqlx::query_as(
            r#"
            SELECT *
            FROM replies AS r
            JOIN replies AS r1
              ON r1.path LIKE r.path || '%'
            WHERE r.id = $1
            ORDER BY r1.created_at
        "#,
        )
        .bind(id.to_bytes())
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(Reply::from).collect())
    }

    async fn create(
        &self,
        post_id: Ulid,
        parent_id: Ulid,
        content: &str,
        user_id: Ulid,
    ) -> Result<Reply, errors::DatabaseError> {
        let reply_id = Ulid::new();
        let reply_row: ReplyRow = sqlx::query_as(
            r#"
                WITH parent_path AS (
                    SELECT 
                        CASE WHEN $2 = $1 
                            THEN CONCAT('/', encode($1, 'hex'), '/')
                            ELSE (SELECT path FROM replies WHERE id = $2)
                        END AS path
                )
                INSERT INTO replies (id, post_id, user_id, reply_to_id, content, path)
                SELECT 
                    $3, $1, $4, $2, $5,
                    CONCAT(parent_path.path, encode($3, 'hex'), '/')
                FROM parent_path
                RETURNING *
            "#,
        )
        .bind(post_id.to_bytes())
        .bind(parent_id.to_bytes())
        .bind(reply_id.to_bytes())
        .bind(user_id.to_bytes())
        .bind(content)
        .fetch_one(&self.pool)
        .await?;

        Ok(Reply::from(reply_row))
    }

    async fn update(&self, id: &Ulid, content: &str) -> Result<Reply, errors::DatabaseError> {
        let id_bytes = id.to_bytes();
        let reply_row: ReplyRow = sqlx::query_as(
            r#"
            UPDATE replies
            SET content = $2
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id_bytes)
        .bind(content)
        .fetch_one(&self.pool)
        .await?;

        Ok(Reply::from(reply_row))
    }

    async fn delete(&self, id: &Ulid) -> Result<(), errors::DatabaseError> {
        let result = sqlx::query(
            r#"
            DELETE FROM replies
            WHERE id = $1
            "#,
        )
        .bind(id.to_bytes())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() > 0 {
            return Ok(());
        }

        Err(errors::DatabaseError::NotFound(String::from(
            "Reply with given id has not been found",
        )))
    }

    async fn delete_all_by_user_id(&self, id: &Ulid) -> Result<Vec<String>, errors::DatabaseError> {
        let bytes = id.to_bytes();

        let deleted_replies_ids: Vec<String> = sqlx::query_scalar(
            r#"
            DELETE FROM replies
            WHERE user_id = $1
            RETURNING id
            "#,
        )
        .bind(bytes)
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|id: [u8; 16]| Ulid::from_bytes(id).to_string())
        .collect();

        Ok(deleted_replies_ids)
    }
}

#[async_trait]
impl Finalizer for PostgresReplyRepository {
    async fn finalize(&self) -> Result<(), errors::AppError> {
        info!("Closing Postgres pool...");
        self.pool.close().await;

        info!("Postgres pool closed");
        Ok(())
    }
}
