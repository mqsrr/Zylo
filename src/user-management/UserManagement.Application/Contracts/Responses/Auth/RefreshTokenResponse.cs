namespace UserManagement.Application.Contracts.Responses.Auth;

public sealed class RefreshTokenResponse
{
    public required string Value { get; init; }

    public required DateTime ExpiresAt { get; init; }
}