using System.Data;
using Dapper;
using UserManagement.Application.Common;
using UserManagement.Application.Repositories.Auth;
using UserManagement.Domain.Auth;
using UserManagement.Domain.Errors;
using UserManagement.Infrastructure.Persistence.Factories;
using UserManagement.Infrastructure.Persistence.Helpers;

namespace UserManagement.Infrastructure.Repositories.Auth;

public sealed class IdentityRepository : IIdentityRepository
{
    private readonly IDbConnectionFactory _dbConnectionFactory;

    public IdentityRepository(IDbConnectionFactory dbConnectionFactory)
    {
        _dbConnectionFactory = dbConnectionFactory;
    }

    public async Task<Result<Identity>> GetByIdAsync(IdentityId id, CancellationToken cancellationToken)
    {
        await using var connection = await _dbConnectionFactory.CreateAsync(cancellationToken);
        var identity = await connection.QueryFirstOrDefaultAsync<Identity>(SqlQueries.Authentication.GetIdentityById, new
        {
            Id = id
        });

        return identity is not null
            ? identity
            : new NotFoundError("Identity not found");
    }

    public async Task<Result<Identity>> GetByUsernameAsync(string username, CancellationToken cancellationToken)
    {
        await using var connection = await _dbConnectionFactory.CreateAsync(cancellationToken);
        var identity = await connection.QueryFirstOrDefaultAsync<Identity>(SqlQueries.Authentication.GetIdentityByUsername, new
        {
            Username = username
        });

        return identity is not null
            ? identity
            : new NotFoundError("Identity not found");
    }

    public async Task<Result> EmailVerifiedAsync(IdentityId id, CancellationToken cancellationToken)
    {
        await using var connection = await _dbConnectionFactory.CreateAsync(cancellationToken);
        int affectedRows = await connection.ExecuteAsync(SqlQueries.Authentication.EmailVerified, new
        {
            Id = id
        });

        return affectedRows > 0
            ? Result.Success()
            : Result.Failure();
    }

    public async Task<Result> CreateAsync(Identity identity, IDbConnection connection, IDbTransaction transaction)
    {
        int createdRows = await connection.ExecuteAsync(SqlQueries.Authentication.Register, identity, transaction);
        return createdRows > 0
            ? Result.Success()
            : Result.Failure();
    }

    public async Task<Result> DeleteByIdAsync(IdentityId id, CancellationToken cancellationToken)
    {
        await using var connection = await _dbConnectionFactory.CreateAsync(cancellationToken);
        int deletedRows = await connection.ExecuteAsync(SqlQueries.Authentication.DeleteById, new
        {
            Id = id
        });

        return deletedRows > 0
            ? Result.Success()
            : new NotFoundError("Identity with given id does not exist.");
    }
}