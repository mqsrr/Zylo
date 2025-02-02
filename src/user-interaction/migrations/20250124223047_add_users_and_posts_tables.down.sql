DROP INDEX IF EXISTS idx_replies_user_id;
DROP INDEX IF EXISTS idx_replies_post_id;

ALTER TABLE replies DROP CONSTRAINT IF EXISTS fk_replies_user;
ALTER TABLE replies DROP CONSTRAINT IF EXISTS fk_replies_posts;

ALTER TABLE replies RENAME COLUMN post_id TO root_id;

DROP TABLE IF EXISTS posts;
DROP TABLE IF EXISTS users;
