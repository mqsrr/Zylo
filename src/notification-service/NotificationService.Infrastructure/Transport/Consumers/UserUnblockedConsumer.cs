using Microsoft.AspNetCore.SignalR;
using Microsoft.Extensions.Logging;
using NotificationService.Application.Abstractions;
using NotificationService.Application.Messages;
using NotificationService.Application.Transport;
using NotificationService.Infrastructure.Hubs;

namespace NotificationService.Infrastructure.Transport.Consumers;

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
        _logger.LogInformation("User {UserId} has unblocked {ReceiverId} user", message.Id, message.BlockedId);
        await _hubContext.Clients.Group(message.BlockedId).UserUnblocked(message.Id);
    }
}