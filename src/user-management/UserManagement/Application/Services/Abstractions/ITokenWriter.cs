using UserManagement.Application.Models;

namespace UserManagement.Application.Services.Abstractions;

public interface ITokenWriter
{
    byte[]? ParseRefreshToken(string? tokenString);
    
    AccessToken GenerateAccessToken(Identity identity);
    
    RefreshToken GenerateRefreshToken(IdentityId id);
}