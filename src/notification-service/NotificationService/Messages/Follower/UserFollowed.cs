namespace NotificationService.Messages.Follower;

public sealed class UserFollowed
{
    public required string Id { get; init; }

    public required string FollowedId { get; init; }
}