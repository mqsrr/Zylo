using Npgsql;
using UserManagement.Application.Attributes;
using UserManagement.Application.Contracts.Requests.Auth;
using UserManagement.Application.Models;
using UserManagement.Application.Services.Abstractions;

namespace UserManagement.Application.Services;

[Decorator]
internal sealed class ExceptionlessAuthService : IAuthService
{
    private readonly IAuthService _authService;
    private readonly ILogger<ExceptionlessAuthService> _logger;

    public ExceptionlessAuthService(IAuthService authService, ILogger<ExceptionlessAuthService> logger)
    {
        _authService = authService;
        _logger = logger;
    }

    public async Task<(AuthenticationResult, RefreshTokenResponse?)> RegisterAsync(RegisterRequest request, CancellationToken cancellationToken)
    {
        try
        {
             return await _authService.RegisterAsync(request, cancellationToken);
        }
        catch (PostgresException e)
        {
            return (new AuthenticationResult
            {
                Success = false,
                Id = null,
                AccessToken = null,
                Error = e.MessageText
            }, null);
        }
    }

    public async Task<(AuthenticationResult, RefreshTokenResponse?)> LoginAsync(LoginRequest request, CancellationToken cancellationToken)
    {
        try
        {
            return await _authService.LoginAsync(request, cancellationToken);
        }
        catch (PostgresException e)
        {
            return (new AuthenticationResult
            {
                Success = false,
                Id = null,
                AccessToken = null,
                Error = e.MessageText
            }, null);
        }
    }

    public async Task<bool> VerifyEmailAsync(IdentityId id, string otpCode, CancellationToken cancellationToken)
    {
        try
        {
            return await _authService.VerifyEmailAsync(id, otpCode, cancellationToken);
        }
        catch (PostgresException e)
        {
            return false;
        }
    }

    public async Task<(AuthenticationResult, RefreshTokenResponse?)> RefreshAccessToken(string? token, CancellationToken cancellationToken)
    {
        try
        {
            return await _authService.RefreshAccessToken(token, cancellationToken);
        }
        catch (PostgresException e)
        {
            return (new AuthenticationResult
            {
                Success = false,
                Id = null,
                AccessToken = null,
                Error = e.MessageText
            }, null);
        }
    }

    public async Task<bool> RevokeRefreshToken(string? token, CancellationToken cancellationToken)
    {
        try
        {
            return await _authService.RevokeRefreshToken(token, cancellationToken);
        }
        catch (PostgresException e)
        {
            _logger.LogError(e, "Revoke refresh token failed");
            return false;
        }
    }

    public async Task<bool> DeleteByIdAsync(IdentityId id, CancellationToken cancellationToken)
    {
        try
        {
            return await _authService.DeleteByIdAsync(id, cancellationToken);
        }
        catch (PostgresException e)
        {
            _logger.LogError(e, "Identity deletion failed");
            return false;
        }
    }
}