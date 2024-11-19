namespace NotificationService.Messages.Post;

public sealed class PostLiked
{
    public required string Id { get; init; }

    public required string UserId { get; init; }
}