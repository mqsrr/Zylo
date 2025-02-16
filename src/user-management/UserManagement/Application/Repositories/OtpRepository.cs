using Dapper;
using UserManagement.Application.Factories.Abstractions;
using UserManagement.Application.Helpers;
using UserManagement.Application.Models;
using UserManagement.Application.Models.Errors;
using UserManagement.Application.Repositories.Abstractions;

namespace UserManagement.Application.Repositories;

public sealed class OtpRepository : IOtpRepository
{
    private readonly IDbConnectionFactory _dbConnectionFactory;

    public OtpRepository(IDbConnectionFactory dbConnectionFactory)
    {
        _dbConnectionFactory = dbConnectionFactory;
    }

    public async Task<Result> CreateAsync(OtpCode code, CancellationToken cancellationToken)
    {
        await using var connection = await _dbConnectionFactory.CreateAsync(cancellationToken);
        int createdRows = await connection.ExecuteAsync(SqlQueries.Authentication.CreateOtpCode, code);

        return createdRows > 0
            ? Result.Success()
            : Result.Failure();
    }

    public async Task<Result<OtpCode>> GetByIdAsync(IdentityId id, CancellationToken cancellationToken)
    {
        await using var connection = await _dbConnectionFactory.CreateAsync(cancellationToken);
        var otpCode = await connection.QueryFirstOrDefaultAsync<OtpCode>(SqlQueries.Authentication.GetOtpCode, new
        {
            Id = id
        });
        
        return otpCode is not null
            ? otpCode
            : new NotFoundError("Otp code not found");
    }

    public async Task<Result> DeleteByIdAsync(IdentityId id, CancellationToken cancellationToken)
    {
        await using var connection = await _dbConnectionFactory.CreateAsync(cancellationToken);
        int deletedRows = await connection.ExecuteAsync(SqlQueries.Authentication.DeleteOtpCodeByIdentityId, new
        {
            Id = id
        });

        return deletedRows > 0
            ? Result.Success()
            : new NotFoundError("There is no attached otp code to the provided id");
    }
}