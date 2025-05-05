using UserManagement.Application.Common;
using UserManagement.Application.Contracts.Requests.Auth;
using UserManagement.Domain.Auth;

namespace UserManagement.Application.Services.Auth;

public interface IAuthService
{
    Task<Result<AuthenticationResult>> RegisterAsync(RegisterRequest request, CancellationToken cancellationToken);

    Task<Result<AuthenticationResult>> LoginAsync(LoginRequest request, CancellationToken cancellationToken);

    Task<Result> VerifyEmailAsync(IdentityId id,string otpCode, CancellationToken cancellationToken);

    Task<Result<AuthenticationResult>> RefreshAccessToken(string? token, CancellationToken cancellationToken);

    Task<Result> RevokeRefreshToken(string? token, CancellationToken cancellationToken);

    Task<Result> DeleteByIdAsync(IdentityId id, CancellationToken cancellationToken);
}