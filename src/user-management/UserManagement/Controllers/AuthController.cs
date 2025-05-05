using Asp.Versioning;
using Microsoft.AspNetCore.Mvc;
using UserManagement.Application.Common;
using UserManagement.Application.Contracts.Requests.Auth;
using UserManagement.Application.Helpers;
using UserManagement.Application.Mappers;
using UserManagement.Application.Services.Auth;
using UserManagement.Domain.Auth;
using UserManagement.Mappers;

namespace UserManagement.Controllers;

[ApiController]
[ApiVersion(1.0)]
public sealed class AuthController : ControllerBase
{
    private const string RefreshTokenCookieName = "refresh_token";
    private readonly IAuthService _authService;

    public AuthController(IAuthService authService)
    {
        _authService = authService;
    }

    [HttpPost(ApiEndpoints.Authentication.Register)]
    public async Task<IActionResult> Register([FromForm] RegisterRequest request, CancellationToken cancellationToken)
    {
        var result = await _authService.RegisterAsync(request, cancellationToken);
        return result.Match<AuthenticationResult, IActionResult>(
            success: authResult => {
                var refreshTokenResponse = authResult.RefreshToken.ToResponse();
                SetHttpOnlyCookie(RefreshTokenCookieName, refreshTokenResponse.Value, refreshTokenResponse.ExpiresAt);
                
                return Ok(authResult.ToResponse());
            },
            failure: error => error.ToProblemObjectResult(HttpContext.TraceIdentifier));

    }

    [HttpPost(ApiEndpoints.Authentication.Login)]
    public async Task<IActionResult> Login([FromBody] LoginRequest request, CancellationToken cancellationToken)
    {
        var result = await _authService.LoginAsync(request, cancellationToken);
        return result.Match<AuthenticationResult, IActionResult>(
            success: authResult => {
                var refreshTokenResponse = authResult.RefreshToken.ToResponse();
                SetHttpOnlyCookie(RefreshTokenCookieName, refreshTokenResponse.Value, refreshTokenResponse.ExpiresAt);
                
                return Ok(authResult.ToResponse());
            },
            failure: error => error.ToProblemObjectResult(HttpContext.TraceIdentifier));

    }

    [HttpPost(ApiEndpoints.Authentication.RefreshAccessToken)]
    public async Task<IActionResult> RefreshAccessToken(CancellationToken cancellationToken)
    {
        var result = await _authService.RefreshAccessToken(Request.Cookies[RefreshTokenCookieName], cancellationToken);
        return result.Match<AuthenticationResult, IActionResult>(
            success: authResult => Ok(authResult.ToResponse()),
            failure: error => error.ToProblemObjectResult(HttpContext.TraceIdentifier));
    }

    [HttpPost(ApiEndpoints.Authentication.RevokeRefreshToken)]
    public async Task<IActionResult> RevokeRefreshToken(CancellationToken cancellationToken)
    {
        var result = await _authService.RevokeRefreshToken(Request.Cookies[RefreshTokenCookieName], cancellationToken);
        return result.Match<IActionResult>(
            success: () => {
                SetHttpOnlyCookie(RefreshTokenCookieName, string.Empty);
                return NoContent();
            },
            failure: error => error.ToProblemObjectResult(HttpContext.TraceIdentifier));
    }

    [HttpPost(ApiEndpoints.Authentication.VerifyUserEmail)]
    public async Task<IActionResult> VerifyEmail([FromRoute] string id, [FromBody] VerifyEmailAddressRequest request, CancellationToken cancellationToken)
    {
        var result = await _authService.VerifyEmailAsync(IdentityId.Parse(id), request.Otp, cancellationToken);
        return result.Match<IActionResult>(
            success: NoContent,
            failure: error => error.ToProblemObjectResult(HttpContext.TraceIdentifier));
    }

    [HttpDelete(ApiEndpoints.Authentication.DeleteIdentity)]
    public async Task<IActionResult> DeleteIdentity([FromRoute] string id, CancellationToken cancellationToken)
    {
        var result = await _authService.DeleteByIdAsync(IdentityId.Parse(id), cancellationToken);
        return result.Match<IActionResult>(
            success: NoContent,
            failure: error => error.ToProblemObjectResult(HttpContext.TraceIdentifier));
    }

    private void SetHttpOnlyCookie(string cookieName, string value, DateTimeOffset? expires = null)
    {
        Response.Cookies.Append(cookieName, value, new CookieOptions
        {
            Expires = expires,
            Secure = true,
            SameSite = SameSiteMode.None,
            HttpOnly = true,
        });
    }
}