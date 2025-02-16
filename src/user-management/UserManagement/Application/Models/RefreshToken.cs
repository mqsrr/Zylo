namespace UserManagement.Application.Models;

public sealed class RefreshToken
{
    public required byte[] Token { get; init; }

    public required IdentityId IdentityId{ get; init; }

    public required DateTime ExpiresAt { get; init; }

    public DateTime CreatedAt { get; init; }
}