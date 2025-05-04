using Microsoft.AspNetCore.SignalR;
using NotificationService.Hubs;
using NotificationService.Services.Abstractions;

namespace NotificationService.Messages.Friend.Consumers;

internal sealed class UserSentFriendRequestConsumer : IConsumer<UserSentFriendRequest>
{
    private readonly IHubContext<NotificationHub, INotificationHub> _hubContext;
    private readonly ILogger<UserSentFriendRequestConsumer> _logger;

    public UserSentFriendRequestConsumer(IHubContext<NotificationHub, INotificationHub> hubContext, ILogger<UserSentFriendRequestConsumer> logger)
    {
        _hubContext = hubContext;
        _logger = logger;
    }

    public async Task ConsumeAsync(UserSentFriendRequest message, CancellationToken cancellationToken)
    {
        _logger.LogInformation("User {} has sent friend request {}", message.Id, message.ReceiverId);
        await _hubContext.Clients.Group(message.ReceiverId).FriendRequestSent(message.Id);
    }
}