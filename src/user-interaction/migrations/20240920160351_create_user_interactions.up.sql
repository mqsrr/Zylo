CREATE TABLE IF NOT EXISTS replies
(
    id          BYTEA PRIMARY KEY,
    reply_to_id BYTEA NOT NULL,
    user_id     BYTEA NOT NULL,
    content     TEXT  NOT NULL,
    created_at  TIMESTAMP DEFAULT NOW()
);


CREATE INDEX IF NOT EXISTS idx_replies_user_id ON replies (user_id);
CREATE INDEX IF NOT EXISTS idx_replies_reply_to_id ON replies (reply_to_id);
