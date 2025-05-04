using NotificationService.Models;

namespace NotificationService.Contracts;

public sealed class DeleteManyByIdRequest
{
    public required IEnumerable<NotificationId> NotificationsIds { get; init; }
}