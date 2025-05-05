using Microsoft.AspNetCore.SignalR;
using Microsoft.Extensions.Logging;
using NotificationService.Application.Abstractions;
using NotificationService.Application.Messages;
using NotificationService.Application.Transport;
using NotificationService.Infrastructure.Hubs;

namespace NotificationService.Infrastructure.Transport.Consumers;

internal sealed class UserFollowedConsumer : IConsumer<UserFollowed>
{
    private readonly IHubContext<NotificationHub, INotificationHub> _hubContext;
    private readonly ILogger<UserFollowedConsumer> _logger;

    public UserFollowedConsumer(IHubContext<NotificationHub, INotificationHub> hubContext, ILogger<UserFollowedConsumer> logger)
    {
        _hubContext = hubContext;
        _logger = logger;
    }

    public async Task ConsumeAsync(UserFollowed message, CancellationToken cancellationToken)
    {
        _logger.LogInformation("User {UserId} has followed {ReceiverId}", message.Id, message.FollowedId);
        await _hubContext.Clients.Group(message.FollowedId).UserFollowed(message.Id);
    }
}