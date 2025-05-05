namespace UserManagement.Domain.Auth;

public sealed class AccessToken
{
    public required string Value { get; init; }

    public required DateTime ExpirationDate { get; init; }
}