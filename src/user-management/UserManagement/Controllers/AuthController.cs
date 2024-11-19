using Asp.Versioning;
using Microsoft.AspNetCore.Mvc;
using UserManagement.Application.Contracts.Requests.Auth;
using UserManagement.Application.Helpers;
using UserManagement.Application.Models;
using UserManagement.Application.Services.Abstractions;

namespace UserManagement.Controllers;

[ApiController]
[ApiVersion(1.0)]
public sealed class AuthController : ControllerBase
{
    private readonly IAuthService _authService;

    public AuthController(IAuthService authService)
    {
        _authService = authService;
    }

    [HttpPost(ApiEndpoints.Authentication.Register)]
    public async Task<IActionResult> Register([FromForm] RegisterRequest request, CancellationToken cancellationToken)
    {
        var (authResult, _) = await _authService.RegisterAsync(request, cancellationToken);
        if (!authResult.Success)
        {
            return BadRequest(authResult);
        }

        return Ok(authResult);
    }

    [HttpPost(ApiEndpoints.Authentication.Login)]
    public async Task<IActionResult> Login([FromBody] LoginRequest request, CancellationToken cancellationToken)
    {
        var (authResult, refreshToken) = await _authService.LoginAsync(request, cancellationToken);
        if (!authResult.Success)
        {
            return BadRequest(authResult);
        }

        if (authResult.Success && refreshToken is null)
        {
            return Ok(authResult);
        }

        SetHttpOnlyCookie("refresh-token", refreshToken!.Value, refreshToken.ExpirationDate);
        return Ok(authResult);
    }

    [HttpPost(ApiEndpoints.Authentication.RefreshAccessToken)]
    public async Task<IActionResult> RefreshAccessToken(CancellationToken cancellationToken)
    {
        var (authResult, refreshToken) = await _authService.RefreshAccessToken(Request.Cookies["refresh-token"], cancellationToken);
        if (!authResult.Success)
        {
            return BadRequest(authResult);
        }

        SetHttpOnlyCookie("refresh-token", refreshToken!.Value, refreshToken.ExpirationDate);
        return Ok(authResult);
    }

    [HttpPost(ApiEndpoints.Authentication.RevokeRefreshToken)]
    public async Task<IActionResult> RevokeRefreshToken(CancellationToken cancellationToken)
    {
        bool isRevoked = await _authService.RevokeRefreshToken(Request.Cookies["refresh-token"], cancellationToken);
        if (!isRevoked)
        {
            return NotFound();
        }

        SetHttpOnlyCookie("refresh-token", string.Empty);
        return NoContent();
    }

    [HttpPost(ApiEndpoints.Authentication.VerifyUserEmail)]
    public async Task<IActionResult> VerifyEmail([FromRoute] string id, [FromBody] VerifyEmailAddressRequest request, CancellationToken cancellationToken)
    {
        bool isVerified = await _authService.VerifyEmailAsync(IdentityId.Parse(id), request.Otp, cancellationToken);
        if (!isVerified)
        {
            return BadRequest("Email verification failed");
        }

        return NoContent();
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