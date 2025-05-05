using Microsoft.AspNetCore.SignalR;
using Microsoft.Extensions.Logging;
using NotificationService.Application.Abstractions;
using NotificationService.Application.Messages;
using NotificationService.Application.Transport;
using NotificationService.Infrastructure.Hubs;

namespace NotificationService.Infrastructure.Transport.Consumers;

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
        _logger.LogInformation("User {UserId} has blocked {ReceiverId} user", message.Id, message.BlockedId);
        await _hubContext.Clients.Group(message.BlockedId).UserBlocked(message.Id);
    }
}