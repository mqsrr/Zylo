using UserManagement.Application.Helpers;
using UserManagement.Application.Models;

namespace UserManagement.Application.Services.Abstractions;

public interface ITokenService
{
    AccessToken GenerateToken(Identity identity);

    Task<Result<RefreshToken>> GetRefreshTokenByIdentityIdAsync(IdentityId id, CancellationToken cancellationToken);

    Task<Result<RefreshToken>> GetRefreshTokenAsync(string refreshToken, CancellationToken cancellationToken);

    Task<Result<RefreshToken>> CreateRefreshTokenAsync(IdentityId id, CancellationToken cancellationToken);

    Task<Result> DeleteRefreshTokenAsync(string refreshToken, CancellationToken cancellationToken);

    Task<Result> DeleteRefreshTokenByIdentityIdAsync(IdentityId id, CancellationToken cancellationToken);
}