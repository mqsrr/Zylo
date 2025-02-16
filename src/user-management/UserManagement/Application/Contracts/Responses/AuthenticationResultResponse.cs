using UserManagement.Application.Models;

namespace UserManagement.Application.Contracts.Responses;

public sealed class AuthenticationResultResponse
{
    public required string Id { get; init; }

    public required AccessToken AccessToken { get; init; }
}