using Microsoft.AspNetCore.SignalR;
using NotificationService.Hubs;
using NotificationService.Services.Abstractions;

namespace NotificationService.Messages.Block.Consumers;

internal sealed class UserUnblockedConsumer : IConsumer<UserUnblocked>
{
    private readonly IHubContext<NotificationHub, INotificationHub> _hubContext;
    private readonly ILogger<UserUnblockedConsumer> _logger;

    public UserUnblockedConsumer(IHubContext<NotificationHub, INotificationHub> hubContext, ILogger<UserUnblockedConsumer> logger)
    {
        _hubContext = hubContext;
        _logger = logger;
    }

    public async Task ConsumeAsync(UserUnblocked message, CancellationToken cancellationToken)
    {
        _logger.LogInformation("User {} has unblocked {} user", message.Id, message.BlockedId);
        await _hubContext.Clients.Group(message.BlockedId).UserUnblocked(message.Id);
    }
}