namespace NotificationService.Application.Messages;

internal sealed class UserUnblocked
{
    public required string Id { get; init; }

    public required string BlockedId { get; init; }
}