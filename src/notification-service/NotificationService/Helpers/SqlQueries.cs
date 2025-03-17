namespace NotificationService.Helpers;

internal static class SqlQueries
{
    public sealed class Authentication
    {
        public const string Register = """
                                       INSERT INTO Identities (id, username, password_hash, password_salt, email_hash, email_salt, email_unique_hash, email_verified) 
                                       VALUES (@Id, @Username, @PasswordHash, @PasswordSalt, @EmailHash, @EmailSalt,@EmailUniqueHash, FALSE);
                                       """;

        public const string GetIdentityByUsername = """
                                                    SELECT id, username, password_hash AS PasswordHash,password_salt AS PasswordSalt,
                                                           email_hash AS EmailHash, email_salt AS EmailSalt,email_unique_hash AS EmailUniqueHash,email_verified AS EmailVerified
                                                    FROM Identities
                                                    WHERE username = @Username;
                                                    """;

        public const string GetIdentityById = """
                                              SELECT id, username, password_hash AS PasswordHash,password_salt AS PasswordSalt, email_hash AS EmailHash, email_salt AS EmailSalt, email_unique_hash AS EmailUniqueHash, email_verified AS EmailVerified
                                              FROM Identities
                                              WHERE id = @Id;
                                              """;

        public const string GetRefreshToken = """
                                              SELECT token AS Token, identity_id AS IdentityId, created_at AS CreatedAt, expires_at AS ExpiresAt
                                              FROM RefreshTokens
                                              WHERE token = @Token;
                                              """;

        public const string GetRefreshTokenByIdentityId = """
                                                          SELECT token AS Token, identity_id AS IdentityId, created_at AS CreatedAt, expires_at AS ExpiresAt
                                                          FROM RefreshTokens
                                                          WHERE identity_id = @IdentityId;
                                                          """;

        public const string CreateRefreshToken = """
                                                 INSERT INTO RefreshTokens (token, expires_at, identity_id)
                                                 VALUES (@Token, @ExpiresAt, @IdentityId);
                                                 """;

        public const string DeleteById = "DELETE FROM Identities WHERE Id = @Id";

        public const string DeleteRefreshToken = "DELETE FROM RefreshTokens WHERE Token = @Token;";

        public const string DeleteAllRefreshTokensById = "DELETE FROM RefreshTokens WHERE Id = @Id;";

        public const string CreateOtpCode = """
                                            INSERT INTO otp (id, code_hash, salt, expires_at)
                                            VALUES (@Id, @CodeHash, @Salt, @ExpiresAt);
                                            """;

        public const string GetOtpCode = """
                                         SELECT id, code_hash AS CodeHash, salt, created_at AS CreatedAt, expires_at AS ExpiresAt FROM otp
                                         WHERE id = @Id;
                                         """;

        public const string DeleteOtpCodeByIdentityId = """
                                                        DELETE FROM otp
                                                        WHERE id = @Id;
                                                        """;

        public const string EmailVerified = """
                                            UPDATE Identities
                                            SET email_verified = true
                                            WHERE id = @Id;
                                            """;
    }

    public sealed class Users
    {
        public const string Create = """
                                     INSERT INTO Users (Id, Name, Username, Bio, Location, BirthDate)
                                     VALUES (@Id, @Name, @Username, @Bio, @Location, @Birthdate)
                                     """;

        public const string GetById = """
                                      SELECT *
                                      FROM Users
                                      WHERE id = @Id;
                                      """;       

        public const string GetUsersSummaryByIds = """
                                       SELECT id, name
                                       FROM Users
                                       WHERE id = ANY(@Ids);
                                       """;

        public const string Update = """
                                     UPDATE Users
                                     SET Name            = @Name,
                                         Bio             = @Bio,
                                         Location        = @Location,
                                         BirthDate       = @Birthdate
                                     WHERE id = @Id
                                     """;
    }
}