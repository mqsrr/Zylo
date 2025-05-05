using Microsoft.Extensions.Logging;
using Npgsql;
using UserManagement.Application.Common;
using UserManagement.Application.Repositories.Auth;
using UserManagement.Domain.Auth;
using UserManagement.Domain.Errors;
using UserManagement.Infrastructure.Extensions;

namespace UserManagement.Infrastructure.Repositories.Auth;

internal sealed class ExceptionlessRefreshTokenRepository : IRefreshTokenRepository
{
    private readonly ILogger<IRefreshTokenRepository> _logger;
    private readonly IRefreshTokenRepository _tokenRepository;

    public ExceptionlessRefreshTokenRepository(IRefreshTokenRepository tokenRepository, ILogger<IRefreshTokenRepository> logger)
    {
        _tokenRepository = tokenRepository;
        _logger = logger;
    }

    public async Task<Result<RefreshToken>> GetByIdentityIdAsync(IdentityId id, CancellationToken cancellationToken)
    {
        try
        {
            return await _tokenRepository.GetByIdentityIdAsync(id, cancellationToken); 
        }
        catch (Exception e)
        {
            _logger.LogError(e, "Unexpected error occured while getting refresh token by identity id: {}", id);
            return new UnexpectedError(e);
        }
    }

    public async Task<Result<RefreshToken>> GetRefreshTokenAsync(byte[] refreshToken, CancellationToken cancellationToken)
    {
        try
        {
            return await _tokenRepository.GetRefreshTokenAsync(refreshToken, cancellationToken); 
        }
        catch (Exception e)
        {
            _logger.LogError(e, "Unexpected error occured while getting refresh token");
            return new UnexpectedError(e);
        }
    }

    public async Task<Result> CreateAsync(RefreshToken refreshToken, CancellationToken cancellationToken)
    {
        try
        {
            return await _tokenRepository.CreateAsync(refreshToken, cancellationToken); 
        }
        catch (PostgresException e) when (e.IsForeignKeyViolation("id"))
        {
            return new BadRequestError("Identity does not exist");
        }
        catch (Exception e)
        {
            _logger.LogError(e, "Unexpected error occured while creating refresh token for identity: {}", refreshToken.IdentityId);
            return new UnexpectedError(e);
        }
    }

    public async Task<Result> DeleteAsync(byte[] refreshToken, CancellationToken cancellationToken)
    {
        try
        {
            return await _tokenRepository.DeleteAsync(refreshToken, cancellationToken); 
        }
        catch (Exception e)
        {
            _logger.LogError(e, "Unexpected error occured during refresh token deletion");
            return new UnexpectedError(e);
        }
    }

    public async Task<Result> DeleteAllByIdAsync(IdentityId id, CancellationToken cancellationToken)
    {
        try
        {
            return await _tokenRepository.DeleteAllByIdAsync(id, cancellationToken); 
        }
        catch (Exception e)
        {
            _logger.LogError(e, "Unexpected error occured while getting all refresh token");
            return new UnexpectedError(e);
        }
    }
}