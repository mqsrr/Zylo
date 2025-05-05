using System.Data;
using Dapper;
using UserManagement.Application.Common;
using UserManagement.Application.Repositories.Users;
using UserManagement.Domain.Errors;
using UserManagement.Domain.Users;
using UserManagement.Infrastructure.Persistence.Factories;
using UserManagement.Infrastructure.Persistence.Helpers;

namespace UserManagement.Infrastructure.Repositories.Users;

internal sealed class UserRepository : IUserRepository
{
    private readonly IDbConnectionFactory _dbConnectionFactory;

    public UserRepository(IDbConnectionFactory dbConnectionFactory)
    {
        _dbConnectionFactory = dbConnectionFactory;
    }

    public async Task<Result<User>> GetByIdAsync(UserId id, CancellationToken cancellationToken)
    {
        await using var connection = await _dbConnectionFactory.CreateAsync(cancellationToken);
        var user = await connection.QueryFirstOrDefaultAsync<User>(SqlQueries.Users.GetById, new
        {
            Id = id
        });

        return user is not null
            ? user
            : new NotFoundError("User not found");
    }

    public async Task<Result<IEnumerable<UserSummary>>> GetBatchUsersSummaryByIds(IEnumerable<UserId> ids, CancellationToken cancellationToken)
    {
        await using var connection = await _dbConnectionFactory.CreateAsync(cancellationToken);
        var result = await connection.QueryAsync<UserSummary>(SqlQueries.Users.GetUsersSummaryByIds, new
        {
            Ids = ids.Select(i => i.Value.ToByteArray()).ToArray()
        });
        
        return Result.Success(result);
    }

    public async Task<Result> CreateAsync(User user, IDbConnection connection, IDbTransaction transaction)
    {
        int affectedRows = await connection.ExecuteAsync(SqlQueries.Users.Create, user, transaction);
        return affectedRows > 0
            ? Result.Success()
            : Result.Failure();
    }

    public async Task<Result> UpdateAsync(User user, CancellationToken cancellationToken)
    {
        await using var connection = await _dbConnectionFactory.CreateAsync(cancellationToken);
        int affectedRows = await connection.ExecuteAsync(SqlQueries.Users.Update, user);

        return affectedRows > 0
            ? Result.Success()
            : Result.Failure();
    }
}