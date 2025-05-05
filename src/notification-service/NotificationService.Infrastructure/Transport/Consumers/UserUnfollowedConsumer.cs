using Microsoft.AspNetCore.SignalR;
using Microsoft.Extensions.Logging;
using NotificationService.Application.Abstractions;
using NotificationService.Application.Messages;
using NotificationService.Application.Transport;
using NotificationService.Infrastructure.Hubs;

namespace NotificationService.Infrastructure.Transport.Consumers;

internal sealed class UserUnfollowedConsumer : IConsumer<UserUnfollowed>
{
    private readonly IHubContext<NotificationHub, INotificationHub> _hubContext;
    private readonly ILogger<UserUnfollowedConsumer> _logger;

    public UserUnfollowedConsumer(IHubContext<NotificationHub, INotificationHub> hubContext, ILogger<UserUnfollowedConsumer> logger)
    {
        _hubContext = hubContext;
        _logger = logger;
    }

    public async Task ConsumeAsync(UserUnfollowed message, CancellationToken cancellationToken)
    {
        _logger.LogInformation("User {UserId} has unfollowed user {ReceiverId}", message.Id, message.FollowedId);
        await _hubContext.Clients.Group(message.FollowedId).UserUnfollowed(message.Id);
    }
}