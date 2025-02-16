CREATE TABLE identities
(
    id                BYTEA               NOT NULL PRIMARY KEY,
    username          VARCHAR(100) UNIQUE NOT NULL,
    password_hash     TEXT                NOT NULL,
    password_salt     TEXT                NOT NULL,
    email_hash        VARCHAR(255)        NOT NULL,
    email_salt        TEXT                NOT NULL,
    email_unique_hash VARCHAR(255) UNIQUE NOT NULL,
    email_verified    BOOLEAN             NOT NULL
);

CREATE TABLE users
(
    id        BYTEA               NOT NULL PRIMARY KEY REFERENCES identities (id) ON DELETE CASCADE,
    name      VARCHAR(100)        NOT NULL,
    username  VARCHAR(100) UNIQUE NOT NULL,
    bio       VARCHAR(500)        NULL,
    location  VARCHAR(255)        NULL,
    birthdate DATE                NOT NULL
);

CREATE TABLE RefreshTokens
(
    token       BYTEA       NOT NULL PRIMARY KEY,
    identity_id BYTEA       NOT NULL REFERENCES identities (Id) ON DELETE CASCADE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at  TIMESTAMPTZ NOT NULL
);

CREATE TABLE otp
(
    id         BYTEA PRIMARY KEY REFERENCES identities (id) ON DELETE CASCADE,
    code_hash  TEXT        NOT NULL,
    salt       TEXT        NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMPTZ NOT NULL
);

CREATE UNIQUE INDEX idx_refresh_tokens_token ON RefreshTokens (token);

CREATE INDEX idx_refresh_tokens_userid ON RefreshTokens (identity_id);

CREATE UNIQUE INDEX idx_identities_id ON Identities (id);

CREATE UNIQUE INDEX idx_identities_username ON Identities (username);

CREATE UNIQUE INDEX idx_users_id ON Users (id);

CREATE INDEX idx_identities_email_salt ON Identities (email_salt);

CREATE INDEX idx_identities_password_salt ON Identities (password_salt);
