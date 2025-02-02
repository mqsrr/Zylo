CREATE TABLE IF NOT EXISTS users
(
    id         BYTEA PRIMARY KEY,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS posts
(
    id         BYTEA PRIMARY KEY,
    user_id    BYTEA NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users (Id) ON DELETE CASCADE
);


ALTER TABLE replies
    ADD CONSTRAINT fk_replies_user FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE;

ALTER TABLE replies
    RENAME COLUMN root_id TO post_id;

ALTER TABLE replies
    ADD CONSTRAINT fk_replies_posts FOREIGN KEY (post_id) REFERENCES posts (id) ON DELETE CASCADE;

CREATE INDEX IF NOT EXISTS idx_replies_user_id ON replies (user_id);
CREATE INDEX IF NOT EXISTS idx_replies_post_id ON replies (post_id);