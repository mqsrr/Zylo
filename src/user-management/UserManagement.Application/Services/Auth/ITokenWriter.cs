using UserManagement.Domain.Auth;

namespace UserManagement.Application.Services.Auth;

public interface ITokenWriter
{
    byte[]? ParseRefreshToken(string? tokenString);

    AccessToken GenerateAccessToken(Identity identity);

    RefreshToken GenerateRefreshToken(IdentityId id);
}