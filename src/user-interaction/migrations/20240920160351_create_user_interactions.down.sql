DROP INDEX IF EXISTS idx_replies_user_id;
DROP INDEX IF EXISTS idx_replies_parent_reply_id;

DROP TABLE IF EXISTS replies CASCADE;