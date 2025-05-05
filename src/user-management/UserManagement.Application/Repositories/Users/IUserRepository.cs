using System.Data;
using UserManagement.Application.Common;
using UserManagement.Domain.Users;

namespace UserManagement.Application.Repositories.Users;

public interface IUserRepository
{
    Task<Result<User>> GetByIdAsync(UserId id, CancellationToken cancellationToken);

    Task<Result<IEnumerable<UserSummary>>> GetBatchUsersSummaryByIds(IEnumerable<UserId> ids, CancellationToken cancellationToken);

    Task<Result> CreateAsync(User user, IDbConnection connection, IDbTransaction transaction);

    Task<Result> UpdateAsync(User user, CancellationToken cancellationToken);
}