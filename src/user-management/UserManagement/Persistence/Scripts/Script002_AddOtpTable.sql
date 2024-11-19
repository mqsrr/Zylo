
CREATE TABLE otp
(
    id         BYTEA PRIMARY KEY,
    code_hash  TEXT        NOT NULL,
    salt       TEXT        NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMPTZ NOT NULL
);

ALTER TABLE Identities
    RENAME COLUMN email TO email_hash;

ALTER TABLE Identities
    RENAME COLUMN passwordhash TO password_hash;

ALTER TABLE Identities
    RENAME COLUMN emailverified TO email_verified;

ALTER TABLE Identities
    ADD COLUMN email_salt    TEXT NOT NULL DEFAULT gen_random_uuid(),
    ADD COLUMN password_salt TEXT NOT NULL DEFAULT gen_random_uuid();


CREATE INDEX idx_identities_emailsalt ON Identities (email_salt);
CREATE INDEX idx_identities_passwordsalt ON Identities (password_salt);