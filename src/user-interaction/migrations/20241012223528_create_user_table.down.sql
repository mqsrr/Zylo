ALTER TABLE replies
    DROP CONSTRAINT IF EXISTS fk_replies_user;

DROP TABLE IF EXISTS users;

DROP INDEX IF EXISTS idx_users_username;