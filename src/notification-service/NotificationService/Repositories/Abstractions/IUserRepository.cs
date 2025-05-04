using NotificationService.Helpers;
using NotificationService.Models;

namespace NotificationService.Repositories.Abstractions;

public interface IUserRepository
{
    Task<Result> CreateAsync(User user, CancellationToken cancellationToken);

    Task<Result> DeleteByIdAsync(UserId id, CancellationToken cancellationToken);
}