using System.Data;
using Dapper;
using UserManagement.Application.Factories.Abstractions;
using UserManagement.Application.Helpers;
using UserManagement.Application.Models;
using UserManagement.Application.Models.Errors;
using UserManagement.Application.Repositories.Abstractions;

namespace UserManagement.Application.Repositories;

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

    public async Task<Result<IEnumerable<User>>> GetBatchByIds(IEnumerable<UserId> ids, CancellationToken cancellationToken)
    {
        await using var connection = await _dbConnectionFactory.CreateAsync(cancellationToken);
        var result = await connection.QueryAsync<User>(SqlQueries.Users.GetByIds, new
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