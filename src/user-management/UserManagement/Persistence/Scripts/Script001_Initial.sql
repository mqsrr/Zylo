CREATE TABLE Identities
(
    Id            BYTEA               NOT NULL PRIMARY KEY,
    Username      VARCHAR(100) UNIQUE NOT NULL,
    PasswordHash  TEXT                NOT NULL,
    Email         VARCHAR(255) UNIQUE NOT NULL,
    EmailVerified BOOLEAN             NOT NULL
);

CREATE TABLE Users
(
    Id                BYTEA        NOT NULL PRIMARY KEY,
    Name              VARCHAR(100) NOT NULL,
    Username          VARCHAR(100) NOT NULL,
    Bio               VARCHAR(500) NULL,
    Location          VARCHAR(255) NULL,
    BirthDate         DATE         NOT NULL
);

CREATE TABLE RefreshTokens
(
    Token          BYTEA       NOT NULL PRIMARY KEY,
    IdentityId     BYTEA       NOT NULL REFERENCES Identities (Id) ON DELETE CASCADE,
    ExpirationDate TIMESTAMPTZ NOT NULL,
    CreatedAt      TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX idx_refresh_tokens_token ON RefreshTokens (Token);

CREATE INDEX idx_refresh_tokens_userid ON RefreshTokens (IdentityId);

CREATE UNIQUE INDEX idx_identities_id ON Identities (Id);

CREATE UNIQUE INDEX idx_identities_username ON Identities (Username);

CREATE UNIQUE INDEX idx_users_id ON Users (Id);
