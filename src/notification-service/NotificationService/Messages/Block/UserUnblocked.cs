namespace NotificationService.Messages.Block;

public sealed class UserUnblocked
{
    public required string Id { get; init; }

    public required string BlockedId { get; init; }
}