using UserManagement.Application.Common;
using UserManagement.Application.Contracts.Requests.Auth;
using UserManagement.Application.Services.Auth;
using UserManagement.Domain.Auth;
using UserManagement.Domain.Errors;

namespace UserManagement.Infrastructure.Services.Auth;

internal sealed class AuthService : IAuthService
{
    private readonly IIdentityService _identityService;
    private readonly IOtpService _otpService;
    private readonly ITokenService _tokenService;

    public AuthService(IIdentityService identityService, ITokenService tokenService, IOtpService otpService)
    {
        _identityService = identityService;
        _tokenService = tokenService;
        _otpService = otpService;
    }

    public async Task<Result<AuthenticationResult>> RegisterAsync(RegisterRequest request, CancellationToken cancellationToken)
    {
        var registerResult = await _identityService.RegisterAsync(request, cancellationToken);
        if (registerResult.IsSuccess is false)
        {
            return registerResult.Error;
        }

        var identity = registerResult.Value!;
        var otpCodeResult = await _otpService.CreateAsync(identity.Id, 6, request.Email, cancellationToken);

        return otpCodeResult.IsSuccess
            ? await GetAuthenticationResultAsync(identity, cancellationToken)
            : otpCodeResult.Error;
    }

    public async Task<Result<AuthenticationResult>> LoginAsync(LoginRequest request, CancellationToken cancellationToken)
    {
        var loginResult = await _identityService.LoginAsync(request.Username, request.Password, cancellationToken);
        return loginResult.IsSuccess
            ? await GetAuthenticationResultAsync(loginResult.Value!, cancellationToken)
            : loginResult.Error;
    }

    public Task<Result> VerifyEmailAsync(IdentityId id, string otpCode, CancellationToken cancellationToken)
    {
        return _identityService.VerifyEmailAsync(id, otpCode, cancellationToken);
    }

    public async Task<Result<AuthenticationResult>> RefreshAccessToken(string? token, CancellationToken cancellationToken)
    {
        if (string.IsNullOrEmpty(token))
        {
            return new BadRequestError("Invalid refresh token");
        }

        var refreshTokenResult = await _tokenService.GetRefreshTokenAsync(token, cancellationToken);
        if (refreshTokenResult.IsSuccess is false)
        {
            return refreshTokenResult.Error;
        }

        var refreshToken = refreshTokenResult.Value!;
        var identityResult = await _identityService.GetByIdAsync(refreshToken.IdentityId, cancellationToken);

        return identityResult.IsSuccess
            ? GetAuthenticationResult(identityResult.Value!, refreshToken)
            : identityResult.Error;
    }

    public Task<Result> RevokeRefreshToken(string? token, CancellationToken cancellationToken)
    {
        return string.IsNullOrEmpty(token) is false
            ? _tokenService.DeleteRefreshTokenAsync(token, cancellationToken)
            : Task.FromResult(Result.Failure(new BadRequestError("Invalid refresh token")));
    }

    public Task<Result> DeleteByIdAsync(IdentityId id, CancellationToken cancellationToken)
    {
        return _identityService.DeleteByIdAsync(id, cancellationToken);
    }

    private async Task<Result<AuthenticationResult>> GetAuthenticationResultAsync(Identity identity, CancellationToken cancellationToken)
    {
        var refreshTokenResult = await _tokenService.GetRefreshTokenByIdentityIdAsync(identity.Id, cancellationToken);
        if (refreshTokenResult.Value is null)
        {
            refreshTokenResult = await _tokenService.CreateRefreshTokenAsync(identity.Id, cancellationToken);
        }

        var refreshToken = refreshTokenResult.Value!;
        return refreshTokenResult.IsSuccess
            ? GetAuthenticationResult(identity, refreshToken)
            : refreshTokenResult.Error;
    }

    private Result<AuthenticationResult> GetAuthenticationResult(Identity identity, RefreshToken refreshToken)
    {
        var accessToken = _tokenService.GenerateToken(identity);
        return new AuthenticationResult
        {
            Id = identity.Id,
            AccessToken = accessToken,
            RefreshToken = refreshToken,
            EmailVerified = identity.EmailVerified
        };
    }
}