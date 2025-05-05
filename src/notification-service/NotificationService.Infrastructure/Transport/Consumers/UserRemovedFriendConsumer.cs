using Microsoft.AspNetCore.SignalR;
using Microsoft.Extensions.Logging;
using NotificationService.Application.Abstractions;
using NotificationService.Application.Messages;
using NotificationService.Application.Transport;
using NotificationService.Infrastructure.Hubs;

namespace NotificationService.Infrastructure.Transport.Consumers;

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
        _logger.LogInformation("User {UserId} has removed the friend {ReceiverId}", message.Id, message.FriendId);
        await _hubContext.Clients.Group(message.FriendId).FriendRemoved(message.Id);
    }
}