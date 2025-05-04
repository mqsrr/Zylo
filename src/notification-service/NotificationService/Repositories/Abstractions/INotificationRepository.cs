using NotificationService.Helpers;
using NotificationService.Models;

namespace NotificationService.Repositories.Abstractions;

public interface INotificationRepository
{
    Task<Result<IEnumerable<Notification>>> GetAllAsync(UserId id, CancellationToken cancellationToken);

    Task<Result> CreateAsync(Notification notification, CancellationToken cancellationToken);

    Task<Result> UpdateSeenAsync(IEnumerable<NotificationId> notificationIds, bool isSeen, CancellationToken cancellationToken);

    Task<Result> DeleteByIdsAsync(IEnumerable<NotificationId> notificationIds, CancellationToken cancellationToken);
}