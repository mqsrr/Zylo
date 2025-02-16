namespace UserManagement.Application.Settings;

public sealed class JwtSettings() : BaseSettings("Jwt")
{
    public required string Audience { get; init; }

    public required string Issuer { get; init; }

    public required string Secret { get; init; }

    public int Expire { get; init; } = 60;
}