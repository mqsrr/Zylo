namespace NotificationService.Application.Messages;

public sealed class UserBlocked
{
    public required string Id { get; init; }

    public required string BlockedId { get; init; }
}