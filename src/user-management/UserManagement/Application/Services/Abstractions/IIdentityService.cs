using UserManagement.Application.Contracts.Requests.Auth;
using UserManagement.Application.Helpers;
using UserManagement.Application.Models;

namespace UserManagement.Application.Services.Abstractions;

public interface IIdentityService
{
    Task<Result<Identity>> GetByIdAsync(IdentityId id, CancellationToken cancellationToken);

    Task<Result<Identity>> RegisterAsync(RegisterRequest request, CancellationToken cancellationToken);

    Task<Result<Identity>> LoginAsync(string username, string password, CancellationToken cancellationToken);

    Task<Result> VerifyEmailAsync(IdentityId id, string otpCode, CancellationToken cancellationToken);

    Task<Result> DeleteByIdAsync(IdentityId id, CancellationToken cancellationToken);
}