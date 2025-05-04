using Microsoft.AspNetCore.SignalR;
using NotificationService.Hubs;
using NotificationService.Services.Abstractions;

namespace NotificationService.Messages.Follower.Consumers;

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
        _logger.LogInformation("User {} has followed {}", message.Id, message.FollowedId);
        await _hubContext.Clients.Group(message.FollowedId).UserFollowed(message.Id);
    }
}