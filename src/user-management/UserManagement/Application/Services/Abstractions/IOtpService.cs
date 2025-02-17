using UserManagement.Application.Helpers;
using UserManagement.Application.Models;

namespace UserManagement.Application.Services.Abstractions;

public interface IOtpService
{
    Task<Result<OtpCode>> CreateAsync(IdentityId id, int length, string email, CancellationToken cancellationToken);

    Task<Result<OtpCode>> GetByIdentityIdAsync(IdentityId id, CancellationToken cancellationToken);

    Task<Result> DeleteByIdentityIdAsync(IdentityId id, CancellationToken cancellationToken);
}