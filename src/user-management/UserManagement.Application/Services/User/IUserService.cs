using System.Data;
using Microsoft.AspNetCore.Http;
using UserManagement.Application.Common;
using UserManagement.Domain.Users;

namespace UserManagement.Application.Services.User;

public interface IUserService
{
    Task<Result<Domain.Users.User>> GetByIdAsync(UserId id, CancellationToken cancellationToken);

    Task<Result<IEnumerable<UserSummary>>> GetBatchUsersSummaryByIdsAsync(IEnumerable<UserId> ids, CancellationToken cancellationToken);

    Task<Result> CreateAsync(Domain.Users.User user, IFormFile profileImage, IFormFile backgroundImage, IDbConnection connection, IDbTransaction transaction,
        CancellationToken cancellationToken);

    Task<Result<Domain.Users.User>> UpdateAsync(Domain.Users.User user, IFormFile? profileImage, IFormFile? backgroundImage, CancellationToken cancellationToken);

    Task<Result> DeleteImagesAsync(UserId id, CancellationToken cancellationToken);
}