using Microsoft.AspNetCore.SignalR;
using NotificationService.Hubs;
using NotificationService.Services.Abstractions;

namespace NotificationService.Messages.Friend.Consumers;

internal sealed class UserRemovedFriendConsumer : IConsumer<UserRemovedFriend>
{
    private readonly IHubContext<NotificationHub, INotificationHub> _hubContext;
    private readonly ILogger<UserRemovedFriendConsumer> _logger;

    public UserRemovedFriendConsumer(IHubContext<NotificationHub, INotificationHub> hubContext, ILogger<UserRemovedFriendConsumer> logger)
    {
        _hubContext = hubContext;
        _logger = logger;
    }

    public async Task ConsumeAsync(UserRemovedFriend message, CancellationToken cancellationToken)
    {
        _logger.LogInformation("User {} has removed the friend {}", message.Id, message.FriendId);
        await _hubContext.Clients.Group(message.FriendId).FriendRemoved(message.Id);
    }
}