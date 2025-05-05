using UserManagement.Application.Common;
using UserManagement.Domain.Auth;

namespace UserManagement.Application.Repositories.Auth;

public interface IRefreshTokenRepository
{
    Task<Result<RefreshToken>> GetByIdentityIdAsync(IdentityId id, CancellationToken cancellationToken);

    Task<Result<RefreshToken>> GetRefreshTokenAsync(byte[] refreshToken, CancellationToken cancellationToken);

    Task<Result> CreateAsync(RefreshToken refreshToken, CancellationToken cancellationToken);

    Task<Result> DeleteAsync(byte[] refreshToken, CancellationToken cancellationToken);

    Task<Result> DeleteAllByIdAsync(IdentityId id, CancellationToken cancellationToken);
}