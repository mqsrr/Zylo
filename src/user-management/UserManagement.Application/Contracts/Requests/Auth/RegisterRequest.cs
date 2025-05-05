using Microsoft.AspNetCore.Http;
using UserManagement.Domain.Auth;

namespace UserManagement.Application.Contracts.Requests.Auth;

public sealed class RegisterRequest
{
    public IdentityId Id { get; } = IdentityId.NewId();

    public required IFormFile ProfileImage { get; init; }

    public required IFormFile BackgroundImage { get; init; }

    public required string Name { get; init; }

    public required string Username { get; init; }

    public required string Password { get; init; }

    public required string Email { get; init; }

    public required DateOnly BirthDate { get; init; }

    public string? Bio { get; init; }

    public string? Location { get; init; }

    public bool EmailVerified { get; set; } = false;
}