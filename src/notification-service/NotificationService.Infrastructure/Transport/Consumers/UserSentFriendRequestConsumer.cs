using Microsoft.AspNetCore.SignalR;
using Microsoft.Extensions.Logging;
using NotificationService.Application.Abstractions;
using NotificationService.Application.Messages;
using NotificationService.Application.Transport;
using NotificationService.Infrastructure.Hubs;

namespace NotificationService.Infrastructure.Transport.Consumers;

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
        _logger.LogInformation("User {UserId} has sent friend request {ReceiverId}", message.Id, message.ReceiverId);
        await _hubContext.Clients.Group(message.ReceiverId).FriendRequestSent(message.Id);
    }
}