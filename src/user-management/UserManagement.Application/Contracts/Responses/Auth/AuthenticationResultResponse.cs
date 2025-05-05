using UserManagement.Domain.Auth;

namespace UserManagement.Application.Contracts.Responses.Auth;

public sealed class AuthenticationResultResponse
{
    public required string Id { get; init; }

    public required bool EmailVerified { get; init; }

    public required AccessToken AccessToken { get; init; }
}