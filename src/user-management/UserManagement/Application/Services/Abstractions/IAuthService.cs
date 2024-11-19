using UserManagement.Application.Contracts.Requests.Auth;
using UserManagement.Application.Models;

namespace UserManagement.Application.Services.Abstractions;

public interface IAuthService
{
    Task<(AuthenticationResult, RefreshTokenResponse?)> RegisterAsync(RegisterRequest request, CancellationToken cancellationToken);
    
    Task<(AuthenticationResult, RefreshTokenResponse?)> LoginAsync(LoginRequest request, CancellationToken cancellationToken);
    
    Task<bool> VerifyEmailAsync(IdentityId id,string otpCode, CancellationToken cancellationToken);
    
    Task<(AuthenticationResult, RefreshTokenResponse?)> RefreshAccessToken(string? token, CancellationToken cancellationToken);
    
    Task<bool> RevokeRefreshToken(string? token, CancellationToken cancellationToken);
    
    Task<bool> DeleteByIdAsync(IdentityId id, CancellationToken cancellationToken);
}