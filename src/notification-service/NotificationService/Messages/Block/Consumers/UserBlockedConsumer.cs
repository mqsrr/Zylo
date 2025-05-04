using Microsoft.AspNetCore.SignalR;
using NotificationService.Hubs;
using NotificationService.Services.Abstractions;

namespace NotificationService.Messages.Block.Consumers;

internal sealed class UserBlockedConsumer : IConsumer<UserBlocked>
{
    private readonly IHubContext<NotificationHub, INotificationHub> _hubContext;
    private readonly ILogger<UserBlockedConsumer> _logger;

    public UserBlockedConsumer(IHubContext<NotificationHub, INotificationHub> hubContext, ILogger<UserBlockedConsumer> logger)
    {
        _hubContext = hubContext;
        _logger = logger;
    }


    public async Task ConsumeAsync(UserBlocked message, CancellationToken cancellationToken)
    {
        _logger.LogInformation("User {} has blocked {} user", message.Id, message.BlockedId);
        await _hubContext.Clients.Group(message.BlockedId).UserBlocked(message.Id);
    }
}