namespace NotificationService.Application.Messages;

internal sealed class UserUnfollowed
{
    public required string Id { get; init; }

    public required string FollowedId { get; init; }
}