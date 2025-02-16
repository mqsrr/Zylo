using UserManagement.Application.Helpers;
using UserManagement.Application.Models;

namespace UserManagement.Application.Repositories.Abstractions;

public interface IOtpRepository
{
    Task<Result<OtpCode>> GetByIdAsync(IdentityId id, CancellationToken cancellationToken);

    Task<Result> CreateAsync(OtpCode code, CancellationToken cancellationToken);

    Task<Result> DeleteByIdAsync(IdentityId id, CancellationToken cancellationToken);
}