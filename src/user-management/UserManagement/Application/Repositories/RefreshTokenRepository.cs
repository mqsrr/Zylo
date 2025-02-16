using Dapper;
using UserManagement.Application.Factories.Abstractions;
using UserManagement.Application.Helpers;
using UserManagement.Application.Models;
using UserManagement.Application.Models.Errors;
using UserManagement.Application.Repositories.Abstractions;

namespace UserManagement.Application.Repositories;

public sealed class RefreshTokenRepository : IRefreshTokenRepository
{
    private readonly IDbConnectionFactory _dbConnectionFactory;

    public RefreshTokenRepository(IDbConnectionFactory dbConnectionFactory)
    {
        _dbConnectionFactory = dbConnectionFactory;
    }

    public async Task<Result<RefreshToken>> GetByIdentityIdAsync(IdentityId id, CancellationToken cancellationToken)
    {
        await using var connection = await _dbConnectionFactory.CreateAsync(cancellationToken);
        var refreshToken = await connection.QueryFirstOrDefaultAsync<RefreshToken>(SqlQueries.Authentication.GetRefreshTokenByIdentityId, new
        {
            IdentityId = id
        });

        return refreshToken is not null
            ? refreshToken
            : new NotFoundError("Refresh token not found");
    }

    public async Task<Result<RefreshToken>> GetRefreshTokenAsync(byte[] refreshToken, CancellationToken cancellationToken)
    {
        await using var connection = await _dbConnectionFactory.CreateAsync(cancellationToken);
        var token = await connection.QueryFirstOrDefaultAsync<RefreshToken>(SqlQueries.Authentication.GetRefreshToken, new
        {
            Token = refreshToken
        });

        return token is not null
            ? token
            : new NotFoundError("Refresh token not found");
    }

    public async Task<Result> CreateAsync(RefreshToken refreshToken, CancellationToken cancellationToken)
    {
        await using var connection = await _dbConnectionFactory.CreateAsync(cancellationToken);
        int createdRows = await connection.ExecuteAsync(SqlQueries.Authentication.CreateRefreshToken, refreshToken);

        return createdRows > 0
            ? Result.Success()
            : Result.Failure();
    }

    public async Task<Result> DeleteAsync(byte[] refreshToken, CancellationToken cancellationToken)
    {
        await using var connection = await _dbConnectionFactory.CreateAsync(cancellationToken);
        int deletedRows = await connection.ExecuteAsync(SqlQueries.Authentication.DeleteRefreshToken, new
        {
            Token = refreshToken
        });

        return deletedRows > 0
            ? Result.Success()
            : new NotFoundError("Refresh token could not be found.");
    }

    public async Task<Result> DeleteAllByIdAsync(IdentityId id, CancellationToken cancellationToken)
    {
        await using var connection = await _dbConnectionFactory.CreateAsync(cancellationToken);
        int deletedRows = await connection.ExecuteAsync(SqlQueries.Authentication.DeleteAllRefreshTokensById, new
        {
            Id = id
        });

        return deletedRows > 0
            ? Result.Success()
            : new NotFoundError("No attached refresh tokens were found.");
    }
}