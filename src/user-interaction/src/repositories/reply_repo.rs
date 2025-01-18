use crate::errors;
use crate::models::reply::Reply;
use crate::models::Finalizer;
use crate::settings::Database;
use async_trait::async_trait;
use chrono::NaiveDateTime;
use sqlx::postgres::PgPoolOptions;
use sqlx::{migrate, PgPool};
use std::collections::HashMap;
use tracing::{info};
use ulid::Ulid;

#[async_trait]
pub trait ReplyRepository: Send + Sync + Finalizer {
    async fn get_all_from_post(&self, post_id: &Ulid) -> Result<Vec<Reply>, errors::DatabaseError>;

    async fn get_all_from_posts(
        &self,
        post_ids: &[Ulid],
    ) -> Result<HashMap<Ulid, Vec<Reply>>, errors::DatabaseError>;

    async fn get_reply_path(&self, id: &Ulid) -> Result<String, errors::DatabaseError>;
    async fn get_with_nested_by_path_prefix(
        &self,
        prefix: &str,
    ) -> Result<Vec<Reply>, errors::DatabaseError>;
    async fn get_with_nested(&self, id: &Ulid) -> Result<Vec<Reply>, errors::DatabaseError>;
    async fn create(&self, reply: &Reply) -> Result<(), errors::DatabaseError>;
    async fn update(&self, id: &Ulid, content: &str) -> Result<Reply, errors::DatabaseError>;
    async fn delete(&self, id: &Ulid) -> Result<(), errors::DatabaseError>;
    async fn delete_all_by_user_id(&self, id: &Ulid) -> Result<Vec<String>, errors::DatabaseError>;
    async fn delete_all_by_post_id(&self, post_id: &Ulid) -> Result<(), errors::DatabaseError>;
}

pub struct PostgresReplyRepository {
    pub pool: PgPool,
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct ReplyRow {
    pub id: Vec<u8>,
    pub root_id: Vec<u8>,
    pub user_id: Vec<u8>,
    pub reply_to_id: Vec<u8>,
    pub content: String,
    pub created_at: NaiveDateTime,
    pub path: String,
}

impl From<ReplyRow> for Reply {
    fn from(row: ReplyRow) -> Self {
        Reply {
            id: Ulid::from_bytes(row.id.try_into().unwrap()),
            root_id: Ulid::from_bytes(row.root_id.try_into().unwrap()),
            reply_to_id: Ulid::from_bytes(row.reply_to_id.try_into().unwrap()),
            user_id: Ulid::from_bytes(row.user_id.try_into().unwrap()),
            content: row.content,
            created_at: row.created_at,
            path: row.path,
        }
    }
}

impl PostgresReplyRepository {
    pub async fn new(config: &Database) -> Result<PostgresReplyRepository, errors::DatabaseError> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&config.uri)
            .await
            .map_err(|e| errors::DatabaseError::PoolCreationError(e.to_string()))?;

        migrate!().run(&pool).await?;
        Ok(Self { pool })
    }
}

#[async_trait]
impl ReplyRepository for PostgresReplyRepository {
    async fn get_all_from_post(&self, post_id: &Ulid) -> Result<Vec<Reply>, errors::DatabaseError> {
        let mut transaction = self.pool.begin().await?;
        let rows: Vec<ReplyRow> = sqlx::query_as(
            r#"
            SELECT
              id,
              root_id,
              reply_to_id,
              user_id,
              content,
              created_at,
              path
            FROM replies
            WHERE root_id = $1
            ORDER BY created_at ASC
            "#,
        )
        .bind(&post_id.to_bytes())
        .fetch_all(&mut *transaction)
        .await?;

        transaction.commit().await?;
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

        let mut transaction = self.pool.begin().await?;
        let rows: Vec<ReplyRow> = sqlx::query_as(
            r#"
            SELECT
              id,
              root_id,
              reply_to_id,
              user_id,
              content,
              created_at,
              path
            FROM replies
            WHERE root_id = ANY($1)
            ORDER BY created_at ASC
            "#,
        )
        .bind(&post_id_bytes)
        .fetch_all(&mut *transaction)
        .await?;

        transaction.commit().await?;
        let all_replies: Vec<Reply> = rows.into_iter().map(Reply::from).collect();

        let mut grouped_map: HashMap<Ulid, Vec<Reply>> = HashMap::new();
        for reply in all_replies {
            grouped_map.entry(reply.root_id).or_default().push(reply);
        }

        Ok(grouped_map)
    }

    async fn get_reply_path(&self, id: &Ulid) -> Result<String, errors::DatabaseError> {
        let mut transaction = self.pool.begin().await?;
        let path: Option<String> = sqlx::query_scalar(
            r#"
            SELECT
              path
            FROM replies
            WHERE id = $1
            "#,
        )
        .bind(&id.to_bytes())
        .fetch_optional(&mut *transaction)
        .await?;
        
        let path = path.ok_or_else(|| errors::DatabaseError::NotFound(String::from(
            "Could not find reply with given id",
        )))?;
        
        transaction.commit().await?;
        Ok(path)
    }

    async fn get_with_nested_by_path_prefix(
        &self,
        prefix: &str,
    ) -> Result<Vec<Reply>, errors::DatabaseError> {
        let mut transaction = self.pool.begin().await?;
        let rows: Vec<ReplyRow> = sqlx::query_as(
            r#"
            SELECT
              id,
              root_id,
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
        .fetch_all(&mut *transaction)
        .await?;

        transaction.commit().await?;
        Ok(rows.into_iter().map(Reply::from).collect())
    }

    async fn get_with_nested(&self, id: &Ulid) -> Result<Vec<Reply>, errors::DatabaseError> {
        let mut transaction = self.pool.begin().await?;
        let row: Option<ReplyRow> = sqlx::query_as(
            r#"
            SELECT
              id,
              root_id",
              reply_to_id",
              user_id",
              content",
              created_at",
              path"
            FROM replies
            WHERE id = $1
            "#,
        )
        .bind(&id.to_bytes())
        .fetch_optional(&mut *transaction)
        .await?;

        let row = row.ok_or_else(|| errors::DatabaseError::NotFound(String::from(
            "Could not find reply with given id",
        )))?;

        let prefix = row.path.clone();
        let nested_rows: Vec<ReplyRow> = sqlx::query_as(
            r#"
            SELECT
              id",
              root_id",
              reply_to_id",
              user_id",
              content",
              created_at",
              path"
            FROM replies
            WHERE path LIKE $1 || '%'
            ORDER BY created_at
            "#,
        )
        .bind(prefix)
        .fetch_all(&mut *transaction)
        .await?;

        transaction.commit().await?;
        Ok(nested_rows.into_iter().map(Reply::from).collect())
    }

    async fn create(&self, reply: &Reply) -> Result<(), errors::DatabaseError> {
        let mut transaction = self.pool.begin().await?;
        sqlx::query(
            r#"
            INSERT INTO replies (id, root_id, user_id, reply_to_id, content, path)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(&reply.id.to_bytes())
        .bind(&reply.root_id.to_bytes())
        .bind(&reply.user_id.to_bytes())
        .bind(&reply.reply_to_id.to_bytes())
        .bind(&reply.content)
        .bind(&reply.path)
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;
        Ok(())
    }

    async fn update(&self, id: &Ulid, content: &str) -> Result<Reply, errors::DatabaseError> {
        let mut transaction = self.pool.begin().await?;
        let id_bytes = id.to_bytes();

        let query_result = sqlx::query(
            r#"
            UPDATE replies
            SET content = $2
            WHERE id = $1
            "#,
        )
        .bind(&id_bytes)
        .bind(content)
        .execute(&mut *transaction)
        .await?;

        if query_result.rows_affected() < 1 {
            return Err(errors::DatabaseError::NotFound(String::from(
                "Could not find reply with given id",
            )));
        }

        let reply_row: ReplyRow = sqlx::query_as(
            r#"
            SELECT
              r.id",
              r.root_id",
              r.user_id",
              r.reply_to_id",
              r.content",
              r.created_at",
              r.path"
            FROM replies r
            WHERE r.id = $1
            "#,
        )
        .bind(&id_bytes)
        .fetch_one(&mut *transaction)
        .await?;

        transaction.commit().await?;
        Ok(Reply::from(reply_row))
    }

    async fn delete(&self, id: &Ulid) -> Result<(), errors::DatabaseError> {
        let mut transaction = self.pool.begin().await?;
        let row: Option<String> = sqlx::query_scalar(r#"SELECT path FROM replies WHERE id = $1"#)
            .bind(&id.to_bytes())
            .fetch_optional(&mut *transaction)
            .await?;

        if row.is_none() {
            return Err(errors::DatabaseError::NotFound(String::from(
                "Could not find reply with provided id",
            )));
        }

        sqlx::query(
            r#"
            DELETE FROM replies
            WHERE path LIKE $1 || '%'
            "#,
        )
        .bind(row)
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;
        Ok(())
    }

    async fn delete_all_by_user_id(&self, id: &Ulid) -> Result<Vec<String>, errors::DatabaseError> {
        let bytes = id.to_bytes();
        let mut transaction = self.pool.begin().await?;

        let deleted_replies_ids: Vec<String> = sqlx::query_scalar(
            r#"
            DELETE FROM replies
            WHERE user_id = $1
            RETURNING id
            "#,
        )
        .bind(&bytes)
        .fetch_all(&mut *transaction)
        .await?
        .into_iter()
        .map(|id: [u8; 16]| Ulid::from_bytes(id.try_into().unwrap()).to_string())
        .collect();

        transaction.commit().await?;
        Ok(deleted_replies_ids)
    }

    async fn delete_all_by_post_id(&self, post_id: &Ulid) -> Result<(), errors::DatabaseError> {
        let bytes = post_id.to_bytes();
        let mut transaction = self.pool.begin().await?;

        sqlx::query(
            r#"
            DELETE FROM replies
            WHERE root_id = $1
            "#,
        )
        .bind(&bytes)
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;
        Ok(())
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