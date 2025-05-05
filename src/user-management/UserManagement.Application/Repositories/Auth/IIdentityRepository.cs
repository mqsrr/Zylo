using System.Data;
using UserManagement.Application.Common;
using UserManagement.Domain.Auth;

namespace UserManagement.Application.Repositories.Auth;

public interface IIdentityRepository
{
    Task<Result<Identity>> GetByIdAsync(IdentityId id, CancellationToken cancellationToken);

    Task<Result<Identity>> GetByUsernameAsync(string username, CancellationToken cancellationToken);

    Task<Result> EmailVerifiedAsync(IdentityId id, CancellationToken cancellationToken);

    Task<Result> CreateAsync(Identity identity, IDbConnection connection, IDbTransaction transaction);

    Task<Result> DeleteByIdAsync(IdentityId id, CancellationToken cancellationToken);
}