using Microsoft.AspNetCore.SignalR;
using Microsoft.Extensions.Logging;
using NotificationService.Application.Abstractions;
using NotificationService.Application.Messages;
using NotificationService.Application.Transport;
using NotificationService.Infrastructure.Hubs;

namespace NotificationService.Infrastructure.Transport.Consumers;

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
        _logger.LogInformation("User {UserId} has declined the friend request from {ReceiverId}", message.Id, message.ReceiverId);
        await _hubContext.Clients.Group(message.ReceiverId).FriendRequestDeclined(message.Id);
    }
}