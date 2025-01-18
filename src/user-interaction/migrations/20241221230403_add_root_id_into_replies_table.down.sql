DROP INDEX IF EXISTS idx_replies_root_id;

ALTER TABLE replies
    DROP COLUMN IF EXISTS root_id;

