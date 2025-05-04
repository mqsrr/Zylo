using NotificationService.Models;

namespace NotificationService.Contracts;

public sealed class UpdateManySeenRequest
{
    public required IEnumerable<NotificationId> NotificationsIds { get; init; }

    public required bool IsSeen { get; init; }
}