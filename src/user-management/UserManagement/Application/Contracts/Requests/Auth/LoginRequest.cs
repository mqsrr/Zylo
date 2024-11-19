namespace UserManagement.Application.Contracts.Requests.Auth;

public sealed class LoginRequest
{
    public required string Username { get; init; }

    public required string Password { get; init; }
}