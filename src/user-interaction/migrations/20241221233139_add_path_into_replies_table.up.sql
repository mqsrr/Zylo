-- Add up migration script here
CREATE EXTENSION IF NOT EXISTS pg_trgm;

ALTER TABLE replies
    ADD COLUMN IF NOT EXISTS path TEXT;

WITH RECURSIVE path_chain AS (
    SELECT
        r.id,
        r.reply_to_id,
        '/' || encode(r.id, 'hex') || '/' AS c_path
    FROM replies r
             LEFT JOIN replies p ON p.id = r.reply_to_id
    WHERE p.id IS NULL

    UNION ALL

    SELECT
        c.id,
        c.reply_to_id,
        path_chain.c_path || encode(c.id, 'hex') || '/'
    FROM replies c
             JOIN path_chain ON path_chain.id = c.reply_to_id
)
UPDATE replies
SET path = path_chain.c_path
FROM path_chain
WHERE replies.id = path_chain.id
  AND replies.path IS NULL;

UPDATE replies
SET path = '/' || encode(id, 'hex') || '/'
WHERE path IS NULL;

ALTER TABLE replies
    ALTER COLUMN path SET NOT NULL;

CREATE INDEX IF NOT EXISTS idx_replies_path_trgm
    ON replies
        USING gin (path gin_trgm_ops);
