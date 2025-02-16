using UserManagement.Application.Helpers;
using UserManagement.Application.Models;

namespace UserManagement.Application.Repositories.Abstractions;

public interface IRefreshTokenRepository
{
    Task<Result<RefreshToken>> GetByIdentityIdAsync(IdentityId id, CancellationToken cancellationToken);

    Task<Result<RefreshToken>> GetRefreshTokenAsync(byte[] refreshToken, CancellationToken cancellationToken);

    Task<Result> CreateAsync(RefreshToken refreshToken, CancellationToken cancellationToken);

    Task<Result> DeleteAsync(byte[] refreshToken, CancellationToken cancellationToken);

    Task<Result> DeleteAllByIdAsync(IdentityId id, CancellationToken cancellationToken);
}