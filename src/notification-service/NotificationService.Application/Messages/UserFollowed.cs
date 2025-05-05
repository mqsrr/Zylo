namespace NotificationService.Application.Messages;

internal sealed class UserFollowed
{
    public required string Id { get; init; }

    public required string FollowedId { get; init; }
}