namespace UserManagement.Application.Helpers;

internal static class SqlQueries
{
    public sealed class Authentication
    {
        public const string Register = """
                                       INSERT INTO Identities (id, username, password_hash, password_salt, email_hash, email_salt, email_verified) 
                                       VALUES (@Id, @Username, @PasswordHash, @PasswordSalt, @EmailHash, @EmailSalt, FALSE);
                                       """;

        public const string GetIdentityByUsername = """
                                                    SELECT id, username, password_hash AS PasswordHash,password_salt AS PasswordSalt, email_hash AS EmailHash, email_salt AS EmailSalt, email_verified AS EmailVerified
                                                    FROM Identities
                                                    WHERE Username = @Username;
                                                    """;

        public const string GetIdentityById = """
                                              SELECT id, username, password_hash AS PasswordHash,password_salt AS PasswordSalt, email_hash AS EmailHash, email_salt AS EmailSalt, email_verified AS EmailVerified
                                              FROM Identities
                                              WHERE Id = @Id;
                                              """;

        public const string GetRefreshToken = """
                                              SELECT *
                                              FROM RefreshTokens
                                              WHERE Token = @Token;
                                              """;

        public const string GetRefreshTokenByIdentityId = """
                                                          SELECT *
                                                          FROM RefreshTokens
                                                          WHERE IdentityId = @IdentityId;
                                                          """;

        public const string CreateRefreshToken = """
                                                 INSERT INTO RefreshTokens (Token, ExpirationDate, IdentityId)
                                                 VALUES (@Token, @ExpirationDate, @IdentityId);
                                                 """;

        public const string DeleteById = "DELETE FROM Identities WHERE Id = @Id";

        public const string DeleteRefreshTokenById = "DELETE FROM RefreshTokens WHERE Token = @Token";

        public const string CreateOtpCode = """
                                            INSERT INTO otp (id, code_hash, salt, expires_at)
                                            VALUES (@Id, @CodeHash, @Salt, @ExpiresAt);
                                            """;

        public const string GetOtpCode = """
                                         SELECT id, code_hash AS CodeHash, salt, created_at AS CreatedAt, expires_at AS ExpiresAt FROM otp
                                         WHERE id = @Id;
                                         """;

        public const string DeleteOtpCode = """
                                            DELETE FROM otp
                                            WHERE Id = @Id;
                                            """;

        public const string EmailVerified = """
                                            UPDATE Identities
                                            SET email_verified = true
                                            WHERE Id = @Id;
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
                                      WHERE Id = @Id;
                                      """;

        public const string Update = """
                                     UPDATE Users
                                     SET Name            = @Name,
                                         Bio             = @Bio,
                                         Location        = @Location,
                                         BirthDate       = @Birthdate
                                     WHERE Id = @Id
                                     """;

        public const string DeleteById = "DELETE FROM Users WHERE Id = @Id";
    }
}