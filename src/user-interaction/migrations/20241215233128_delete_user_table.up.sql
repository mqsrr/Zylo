-- Add up migration script here
ALTER TABLE replies
    DROP CONSTRAINT fk_replies_user;

DROP INDEX IF EXISTS idx_users_username CASCADE;

DROP TABLE IF EXISTS users;

