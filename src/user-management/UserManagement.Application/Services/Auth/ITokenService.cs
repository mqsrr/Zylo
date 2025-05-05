using UserManagement.Application.Common;
using UserManagement.Domain.Auth;

namespace UserManagement.Application.Services.Auth;

public interface ITokenService
{
    AccessToken GenerateToken(Identity identity);

    Task<Result<RefreshToken>> GetRefreshTokenByIdentityIdAsync(IdentityId id, CancellationToken cancellationToken);

    Task<Result<RefreshToken>> GetRefreshTokenAsync(string refreshToken, CancellationToken cancellationToken);

    Task<Result<RefreshToken>> CreateRefreshTokenAsync(IdentityId id, CancellationToken cancellationToken);

    Task<Result> DeleteRefreshTokenAsync(string refreshToken, CancellationToken cancellationToken);
}