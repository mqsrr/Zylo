namespace NotificationService.Messages.Friend;

public sealed class UserRemovedFriend
{
    public required string Id { get; init; }
    
    public required string FriendId { get; init; }
}