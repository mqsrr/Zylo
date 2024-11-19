use chrono::NaiveDateTime;
use sqlx::PgPool;
use ulid::Ulid;
use crate::errors::AppError;
use crate::models::user::User;
use crate::services::user_profile::{ProfilePictureService, UserProfileService};

#[derive(Debug, Clone)]
pub struct Reply {
    pub id: Ulid,
    pub user: User,
    pub reply_to_id: Ulid,
    pub content: String,
    pub created_at: NaiveDateTime,
}
#[derive(Debug, Clone, sqlx::FromRow)]
struct ReplyRow {
    pub id: Vec<u8>,
    pub user_id: Vec<u8>,
    pub username: String,
    pub name: String,
    pub bio: Option<String>,
    pub location: Option<String>,
    pub reply_to_id: Vec<u8>,
    pub content: String,
    pub created_at: NaiveDateTime,
}

impl From<ReplyRow> for Reply {
    fn from(row: ReplyRow) -> Self {
        Reply {
            id: Ulid::from_bytes(row.id.try_into().unwrap()),
            user: User {
                id: Ulid::from_bytes(row.user_id.try_into().unwrap()),
                username: row.username,
                name: row.name,
                bio: row.bio,
                location: row.location,
                profile_image: None,
            },
            reply_to_id: Ulid::from_bytes(row.reply_to_id.try_into().unwrap()),
            content: row.content,
            created_at: row.created_at,
        }
    }
}

impl Reply {
    pub async fn get_all_from_post(pool: &PgPool, id: Ulid, user_profile_service: &mut UserProfileService) -> Result<Vec<Reply>, AppError> {
        let records: Vec<ReplyRow> = sqlx::query_as!(
            ReplyRow,
            r#"
            WITH RECURSIVE reply_tree AS (
                SELECT r.id, r.user_id, r.reply_to_id, r.content, r.created_at,
                       u.username, u.name, u.bio, u.location
                FROM replies r
                         JOIN users u ON r.user_id = u.id
                WHERE r.reply_to_id = $1
            
                UNION ALL
            
                SELECT r.id, r.user_id, r.reply_to_id, r.content, r.created_at,
                       u.username, u.name, u.bio, u.location
                FROM replies r
                         JOIN users u ON r.user_id = u.id
                         INNER JOIN reply_tree rt ON rt.id = r.reply_to_id
            )
            SELECT id AS "id!", user_id AS "user_id!", reply_to_id AS "reply_to_id!",
             content AS "content!", created_at AS "created_at!",
              username AS "username!", name AS "name!", bio, location
            FROM reply_tree;
            "#,&id.to_bytes())
            .fetch_all(pool)
            .await?;

        let mut replies: Vec<Reply> = records.into_iter().map(Reply::from).collect();
        for reply in &mut replies {
            reply.user.profile_image = Some(user_profile_service.get_profile_picture(reply.user.id).await?)
        }
        
        Ok(replies)
    }

    pub async fn create(pool: &PgPool, user_profile_service: &mut UserProfileService, data: &Reply) -> Result<Reply, AppError> {
        sqlx::query!(
            r#"
            INSERT INTO replies (id, user_id, reply_to_id, content)
            VALUES ($1, $2, $3, $4)
            "#, &data.id.to_bytes(), &data.user.id.to_bytes(), &data.reply_to_id.to_bytes(), data.content)
            .execute(pool)
            .await?;
        
        let reply_row = sqlx::query_as!(ReplyRow, 
        r#"
        SELECT r.id AS "id!", user_id AS "user_id!", reply_to_id AS "reply_to_id!",
             content AS "content!", created_at AS "created_at!",
              username AS "username!", name AS "name!", bio, location FROM replies r
        JOIN users u ON u.id = r.user_id
        WHERE r.id = $1
        "#, &data.id.to_bytes())
            .fetch_one(pool)
            .await?;
        
        let mut created_reply = Reply::from(reply_row);
        created_reply.user.profile_image = Some(user_profile_service.get_profile_picture(created_reply.user.id).await?);
        
        Ok(created_reply)
    }

    pub async fn update(pool: &PgPool, user_profile_service: &mut UserProfileService, post_id: &Ulid, content: &String) -> Result<Reply, AppError> {
        sqlx::query!(
            r#"
            UPDATE replies
            SET content = $2
            WHERE id=$1
            "#, &post_id.to_bytes(), content)
            .execute(pool)
            .await?;
        
        let reply_row = sqlx::query_as!(ReplyRow, 
        r#"
        SELECT r.id AS "id!", user_id AS "user_id!", reply_to_id AS "reply_to_id!",
             content AS "content!", created_at AS "created_at!",
              username AS "username!", name AS "name!", bio, location FROM replies r
        JOIN users u ON u.id = r.user_id
        WHERE r.id = $1
        "#, &post_id.to_bytes())
            .fetch_one(pool)
            .await?;

        let mut updated_reply = Reply::from(reply_row);
        updated_reply.user.profile_image = Some(user_profile_service.get_profile_picture(updated_reply.user.id).await?);

        Ok(updated_reply)
    }

    pub async fn delete(pool: &PgPool, id: Ulid) -> Result<(), AppError> {
        sqlx::query!(
        r#"
        DELETE FROM replies
        WHERE id = $1 OR reply_to_id = $1
        "#, &id.to_bytes()).execute(pool).await?;

        Ok(())
    }
}

