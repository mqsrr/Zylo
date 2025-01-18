-- Add up migration script here
ALTER TABLE replies
    ADD COLUMN IF NOT EXISTS root_id BYTEA;

WITH RECURSIVE chain AS (
    SELECT
        r.id AS current_id,
        r.reply_to_id AS current_parent,
        r.reply_to_id AS computed_root
    FROM replies r
             LEFT JOIN replies parent ON parent.id = r.reply_to_id
    WHERE parent.id IS NULL

    UNION ALL

    SELECT
        child.id AS current_id,
        child.reply_to_id AS current_parent,
        parent_chain.computed_root
    FROM replies child
             JOIN chain parent_chain ON parent_chain.current_id = child.reply_to_id
)
UPDATE replies
SET root_id = id
WHERE root_id IS NULL;

ALTER TABLE replies
    ALTER COLUMN root_id SET NOT NULL;

CREATE INDEX IF NOT EXISTS idx_replies_root_id
    ON replies (root_id);
