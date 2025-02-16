using System.Data;
using UserManagement.Application.Helpers;
using UserManagement.Application.Models;

namespace UserManagement.Application.Repositories.Abstractions;

public interface IIdentityRepository
{
    Task<Result<Identity>> GetByIdAsync(IdentityId id, CancellationToken cancellationToken);

    Task<Result<Identity>> GetByUsernameAsync(string username, CancellationToken cancellationToken);

    Task<Result> EmailVerifiedAsync(IdentityId id, CancellationToken cancellationToken);

    Task<Result> CreateAsync(Identity identity, IDbConnection connection, IDbTransaction transaction);

    Task<Result> DeleteByIdAsync(IdentityId id, CancellationToken cancellationToken);
}