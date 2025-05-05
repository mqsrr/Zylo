namespace UserManagement.Domain.Auth;

public sealed class AuthenticationResult
{
    public required IdentityId Id { get; init; }

    public required bool EmailVerified { get; init; }

    public required AccessToken AccessToken { get; init; }

    public required RefreshToken RefreshToken { get; init; }
}