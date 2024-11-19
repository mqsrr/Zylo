namespace NotificationService.Messages.Follower;

public sealed class UserUnfollowed
{
    public required string Id { get; init; }

    public required string FollowedId { get; init; }
}