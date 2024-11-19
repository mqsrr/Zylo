
namespace NotificationService.Hubs;

public interface INotificationHub
{
    Task UserBlocked(string blockedUserId);
    Task UserUnblocked(string unblockedUserId);

    Task UserFollowed(string followedUserId);
    Task UserUnfollowed(string unfollowedUserId);

    Task FriendRequestSent(string friendUserId);
    Task FriendRequestAccepted(string friendUserId);
    Task FriendRequestDeclined(string friendUserId);
    Task FriendRemoved(string friendUserId);

    Task PostLiked(string postId);
}