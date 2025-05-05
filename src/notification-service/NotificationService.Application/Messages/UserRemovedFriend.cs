namespace NotificationService.Application.Messages;

internal sealed class UserRemovedFriend
{
    public required string Id { get; init; }
    
    public required string FriendId { get; init; }
}