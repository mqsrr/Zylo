using Microsoft.AspNetCore.SignalR;
using NotificationService.Hubs;
using NotificationService.Services.Abstractions;

namespace NotificationService.Messages.Friend.Consumers;

internal sealed class UserAcceptedFriendRequestConsumer : IConsumer<UserAcceptedFriendRequest>
{
    private readonly IHubContext<NotificationHub, INotificationHub> _hubContext;
    private readonly ILogger<UserAcceptedFriendRequestConsumer> _logger;

    public UserAcceptedFriendRequestConsumer(IHubContext<NotificationHub, INotificationHub> hubContext, ILogger<UserAcceptedFriendRequestConsumer> logger)
    {
        _hubContext = hubContext;
        _logger = logger;
    }

    public async Task ConsumeAsync(UserAcceptedFriendRequest message, CancellationToken cancellationToken)
    {
        _logger.LogInformation("User {UserId} has accepted friend request {ReceiverId}", message.Id, message.ReceiverId);
        await _hubContext.Clients.Group(message.ReceiverId).FriendRequestAccepted(message.Id);
    }
}