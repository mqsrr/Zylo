using System.Data;
using UserManagement.Application.Helpers;
using UserManagement.Application.Models;

namespace UserManagement.Application.Repositories.Abstractions;

public interface IUserRepository
{
    Task<Result<User>> GetByIdAsync(UserId id, CancellationToken cancellationToken);
    
    Task<Result<IEnumerable<UserSummary>>> GetBatchUsersSummaryByIds(IEnumerable<UserId> ids, CancellationToken cancellationToken);

    Task<Result> CreateAsync(User user, IDbConnection connection, IDbTransaction transaction);

    Task<Result> UpdateAsync(User user, CancellationToken cancellationToken);
}