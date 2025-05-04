using Microsoft.AspNetCore.SignalR;
using NotificationService.Hubs;
using NotificationService.Services.Abstractions;

namespace NotificationService.Messages.Follower.Consumers;

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
        _logger.LogInformation("User {} has unfollowed user {}", message.Id, message.FollowedId);
        await _hubContext.Clients.Group(message.FollowedId).UserUnfollowed(message.Id);
    }
}