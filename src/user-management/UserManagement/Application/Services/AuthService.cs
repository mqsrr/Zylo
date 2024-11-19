using System.Data;
using Dapper;
using Mediator;
using Npgsql;
using UserManagement.Application.Contracts.Requests.Auth;
using UserManagement.Application.Factories.Abstractions;
using UserManagement.Application.Helpers;
using UserManagement.Application.Mappers;
using UserManagement.Application.Messaging.Users;
using UserManagement.Application.Models;
using UserManagement.Application.Services.Abstractions;

namespace UserManagement.Application.Services;

internal sealed class AuthService : IAuthService
{
    private readonly IDbConnectionFactory _dbConnectionFactory;
    private readonly ITokenWriter _tokenWriter;
    private readonly IPublisher _publisher;
    private readonly IHashService _hashService;

    public AuthService(IDbConnectionFactory dbConnectionFactory, ITokenWriter tokenWriter, IPublisher publisher, IHashService hashService)
    {
        _dbConnectionFactory = dbConnectionFactory;
        _tokenWriter = tokenWriter;
        _publisher = publisher;
        _hashService = hashService;
    }

    public async Task<(AuthenticationResult, RefreshTokenResponse?)> RegisterAsync(RegisterRequest request, CancellationToken cancellationToken)
    {
        await using var connection = await _dbConnectionFactory.CreateAsync(cancellationToken);
        await using var transaction = await connection.BeginTransactionAsync(IsolationLevel.Serializable, cancellationToken);
        try
        {
            var identity = request.ToIdentity(_hashService);

            int affectedRows = await connection.ExecuteAsync(SqlQueries.Authentication.Register, identity, transaction);
            if (affectedRows < 1)
            {
                return (new AuthenticationResult
                {
                    Success = false,
                    Error = "Failed to register user. Check credentials and try again."
                }, null);
            }

            await transaction.CommitAsync(cancellationToken);
            await _publisher.Publish(new CreateUserNotification
            {
                Request = request
            }, cancellationToken).ConfigureAwait(false);

            return ValueTuple.Create<AuthenticationResult, RefreshTokenResponse?>(new AuthenticationResult
            {
                Id = identity.Id.Value,
                EmailVerified = identity.EmailVerified,
                Success = true,
            }, null);
        }
        catch (PostgresException)
        {
            await transaction.RollbackAsync(cancellationToken);
            throw;
        }
    }

    public async Task<(AuthenticationResult, RefreshTokenResponse?)> LoginAsync(LoginRequest request, CancellationToken cancellationToken)
    {
        await using var connection = await _dbConnectionFactory.CreateAsync(cancellationToken);
        await using var transaction = await connection.BeginTransactionAsync(IsolationLevel.ReadCommitted, cancellationToken);
        try
        {
            var identity = await connection.QueryFirstOrDefaultAsync<Identity>(
                SqlQueries.Authentication.GetIdentityByUsername,
                new {request.Username},
                transaction);

            if (identity is null)
            {
                return (new AuthenticationResult
                {
                    Success = false,
                    Error = "Could not find user with given credentials"
                }, null);
            }

            bool passwordVerificationResult = _hashService.VerifyHash(request.Password, identity.PasswordHash, identity.PasswordSalt);
            if (!passwordVerificationResult)
            {
                return (new AuthenticationResult
                {
                    Success = false,
                    Error = "Incorrect credentials or the email address was not verified!"
                }, null);
            }

            if (!identity.EmailVerified)
            {
                return ValueTuple.Create<AuthenticationResult, RefreshTokenResponse?>(new AuthenticationResult
                {
                    Id = identity.Id.Value,
                    EmailVerified = identity.EmailVerified,
                    Success = true,
                }, null);
            }

            var accessToken = _tokenWriter.GenerateAccessToken(identity);
            var refreshToken = await connection.QueryFirstOrDefaultAsync<RefreshToken>(
                SqlQueries.Authentication.GetRefreshTokenByIdentityId,
                new {IdentityId = identity.Id},
                transaction);

            if (refreshToken is not null)
            {
                return ValueTuple.Create(new AuthenticationResult
                {
                    Id = identity.Id.Value,
                    EmailVerified = identity.EmailVerified,
                    Success = true,
                    AccessToken = accessToken,
                }, refreshToken.ToResponse());
            }

            refreshToken = _tokenWriter.GenerateRefreshToken(identity.Id);
            await connection.ExecuteAsync(
                SqlQueries.Authentication.CreateRefreshToken,
                new {IdentityId = identity.Id, refreshToken.Token, refreshToken.ExpirationDate},
                transaction);

            await transaction.CommitAsync(cancellationToken);
            return ValueTuple.Create(new AuthenticationResult
            {
                Id = identity.Id.Value,
                EmailVerified = identity.EmailVerified,
                Success = true,
                AccessToken = accessToken,
            }, refreshToken.ToResponse());
        }
        catch (PostgresException)
        {
            await transaction.RollbackAsync(cancellationToken);
            throw;
        }
    }

    public async Task<bool> VerifyEmailAsync(IdentityId id, string otpCode, CancellationToken cancellationToken)
    {
        await using var connection = await _dbConnectionFactory.CreateAsync(cancellationToken);
        await using var transaction = await connection.BeginTransactionAsync(IsolationLevel.RepeatableRead, cancellationToken);
        try
        {
            var code = await connection.QueryFirstOrDefaultAsync<OtpCode>(SqlQueries.Authentication.GetOtpCode, new {Id = id}, transaction);
            if (code is null)
            {
                return false;
            }

            bool codeMatched = _hashService.VerifyHash(otpCode, code.CodeHash, code.Salt);
            if (!codeMatched)
            {
                return false;
            }

            await connection.ExecuteAsync(SqlQueries.Authentication.DeleteOtpCode, new {Id = id}, transaction);
            await connection.ExecuteAsync(SqlQueries.Authentication.EmailVerified, new {Id = id}, transaction);
        
            await transaction.CommitAsync(cancellationToken);
            return true;
        }
        catch (PostgresException)
        {
            await transaction.RollbackAsync(cancellationToken);
            throw;
        }
    }

    public async Task<(AuthenticationResult, RefreshTokenResponse?)> RefreshAccessToken(string? token, CancellationToken cancellationToken)
    {
        byte[]? refreshTokenBytes = _tokenWriter.ParseRefreshToken(token);
        if (refreshTokenBytes is null)
        {
            return (new AuthenticationResult
            {
                Success = false,
                Error = "Refresh token is not valid"
            }, null);
        }

        await using var connection = await _dbConnectionFactory.CreateAsync(cancellationToken);
        await using var transaction = await connection.BeginTransactionAsync(IsolationLevel.ReadCommitted, cancellationToken);
        try
        {
            var refreshToken = await connection.QueryFirstOrDefaultAsync<RefreshToken>(
                SqlQueries.Authentication.GetRefreshToken, 
                new {Token = refreshTokenBytes},
                transaction);

            if (refreshToken is null || refreshToken.ExpirationDate < DateTime.UtcNow)
            {
                return (new AuthenticationResult
                {
                    Success = false,
                    Error = "Refresh token is not valid"
                }, null);
            }

            var identity = await connection.QueryFirstAsync<Identity>(
                SqlQueries.Authentication.GetIdentityById,
                new {Id = refreshToken.IdentityId},
                transaction);

            await transaction.CommitAsync(cancellationToken);
            var accessToken = _tokenWriter.GenerateAccessToken(identity);
            
            return (new AuthenticationResult
            {
                Id = identity.Id.Value,
                Success = true,
                AccessToken = accessToken,
            }, refreshToken.ToResponse());
        }
        catch (PostgresException)
        {
            await transaction.RollbackAsync(cancellationToken);
            throw;
        }
    }

    public async Task<bool> RevokeRefreshToken(string? token, CancellationToken cancellationToken)
    {
        byte[]? refreshTokenBytes = _tokenWriter.ParseRefreshToken(token);
        await using var connection = await _dbConnectionFactory.CreateAsync(cancellationToken);
        await using var transaction = await connection.BeginTransactionAsync(IsolationLevel.ReadCommitted, cancellationToken);
        
        try
        {
            int affectedRows = await connection.ExecuteAsync(SqlQueries.Authentication.DeleteRefreshTokenById, new {Token = refreshTokenBytes});
            await transaction.CommitAsync(cancellationToken);
            
            return affectedRows > 0;
        }
        catch (PostgresException)
        {
            await transaction.RollbackAsync(cancellationToken);
            throw;
        }

    }

    public async Task<bool> DeleteByIdAsync(IdentityId id, CancellationToken cancellationToken)
    {
        await using var connection = await _dbConnectionFactory.CreateAsync(cancellationToken);
        await using var transaction = await connection.BeginTransactionAsync(IsolationLevel.RepeatableRead, cancellationToken);
        
        try
        {
            int isDeleted = await connection.ExecuteAsync(SqlQueries.Authentication.DeleteById, new {Id = id});
            await transaction.CommitAsync(cancellationToken);
            return isDeleted > 0;
        }
        catch (PostgresException)
        {
            await transaction.RollbackAsync(cancellationToken);
            throw;
        }

    }
}