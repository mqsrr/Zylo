using Microsoft.AspNetCore.SignalR;
using NotificationService.Hubs;
using NotificationService.Services.Abstractions;

namespace NotificationService.Messages.Friend.Consumers;

internal sealed class UserDeclinedFriendRequestConsumer : IConsumer<UserDeclinedFriendRequest>
{
    private readonly IHubContext<NotificationHub, INotificationHub> _hubContext;
    private readonly ILogger<UserDeclinedFriendRequestConsumer> _logger;

    public UserDeclinedFriendRequestConsumer(IHubContext<NotificationHub, INotificationHub> hubContext, ILogger<UserDeclinedFriendRequestConsumer> logger)
    {
        _hubContext = hubContext;
        _logger = logger;
    }

    public async Task ConsumeAsync(UserDeclinedFriendRequest message, CancellationToken cancellationToken)
    {
        _logger.LogInformation("User {} has declined the friend request from {}", message.Id, message.ReceiverId);
        await _hubContext.Clients.Group(message.ReceiverId).FriendRequestDeclined(message.Id);
    }
}