namespace NotificationService.Application.Messages;

internal sealed class UserSentFriendRequest
{
    public required string Id { get; init; }

    public required string ReceiverId { get; init; }
}