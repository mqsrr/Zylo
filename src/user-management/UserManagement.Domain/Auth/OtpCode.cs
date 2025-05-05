namespace UserManagement.Domain.Auth;

public sealed class OtpCode
{
    public required IdentityId Id { get; init; }

    public required string CodeHash { get; init; }

    public required string Salt { get; init; }

    public required DateTimeOffset CreatedAt { get; init; }

    public required DateTimeOffset ExpiresAt { get; init; }
}