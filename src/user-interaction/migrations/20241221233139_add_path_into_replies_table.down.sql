-- Add down migration script here
DROP INDEX IF EXISTS idx_replies_path_trgm;

ALTER TABLE replies
    DROP COLUMN IF EXISTS path;