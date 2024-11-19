namespace UserManagement.Application.Models;

public sealed class RefreshTokenResponse
{
    public required string Value { get; init; }
    
    public required DateTime ExpirationDate { get; init; }
    
    public bool Revoked { get; init; }
}