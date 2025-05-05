using Dapper;
using UserManagement.Application.Common;
using UserManagement.Application.Repositories.Auth;
using UserManagement.Domain.Auth;
using UserManagement.Domain.Errors;
using UserManagement.Infrastructure.Persistence.Factories;
using UserManagement.Infrastructure.Persistence.Helpers;

namespace UserManagement.Infrastructure.Repositories.Auth;

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