-- Add down migration script here
CREATE TABLE IF NOT EXISTS users
(
    id       BYTEA PRIMARY KEY,
    username VARCHAR(250) UNIQUE NOT NULL,
    name     VARCHAR(250) NOT NULL,
    bio      TEXT         NULL,
    location VARCHAR(250) NULL
    );

CREATE UNIQUE INDEX IF NOT EXISTS idx_users_username ON users(username);

ALTER TABLE replies
    ADD CONSTRAINT fk_replies_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;
