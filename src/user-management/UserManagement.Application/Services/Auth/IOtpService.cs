using UserManagement.Application.Common;
using UserManagement.Domain.Auth;

namespace UserManagement.Application.Services.Auth;

public interface IOtpService
{
    Task<Result<OtpCode>> CreateAsync(IdentityId id, int length, string email, CancellationToken cancellationToken);

    Task<Result<OtpCode>> GetByIdentityIdAsync(IdentityId id, CancellationToken cancellationToken);

    Task<Result> DeleteByIdentityIdAsync(IdentityId id, CancellationToken cancellationToken);
}