using System.Data;
using UserManagement.Application.Helpers;
using UserManagement.Application.Models;

namespace UserManagement.Application.Services.Abstractions;

public interface IUserService
{
    Task<Result<User>> GetByIdAsync(UserId id, CancellationToken cancellationToken);
    
    Task<Result<IEnumerable<UserSummary>>> GetBatchUsersSummaryByIdsAsync(IEnumerable<UserId> ids, CancellationToken cancellationToken);

    Task<Result> CreateAsync(User user, IFormFile profileImage, IFormFile backgroundImage, IDbConnection connection, IDbTransaction transaction,
        CancellationToken cancellationToken);

    Task<Result<User>> UpdateAsync(User user, IFormFile? profileImage, IFormFile? backgroundImage, CancellationToken cancellationToken);

    Task<Result> DeleteImagesAsync(UserId id, CancellationToken cancellationToken);
}