namespace UserManagement.Application.Models;



public sealed class RefreshToken
{
    public required byte[] Token { get; init; }
    
    public required Ulid IdentityId{ get; init; }
    
    public required DateTime ExpirationDate { get; init; }
    
    public bool Revoked { get; init; }
}