using UserManagement.Application.Common;
using UserManagement.Domain.Auth;

namespace UserManagement.Application.Repositories.Auth;

public interface IOtpRepository
{
    Task<Result<OtpCode>> GetByIdAsync(IdentityId id, CancellationToken cancellationToken);

    Task<Result> CreateAsync(OtpCode code, CancellationToken cancellationToken);

    Task<Result> DeleteByIdAsync(IdentityId id, CancellationToken cancellationToken);
}