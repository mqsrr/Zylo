using NotificationService.Models;

namespace NotificationService.Messages.User;

public sealed class UserDeleted
{
    public required UserId Id { get; init; }
}