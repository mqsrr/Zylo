namespace UserManagement.Application.Models;

public sealed class AuthenticationResult
{
    public required bool Success { get; init; }
    
    public Ulid? Id { get; init; }
    
    public bool? EmailVerified { get; init; }

    public AccessToken? AccessToken { get; init; }
    
    public string? Error { get; init; }
}