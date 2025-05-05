using System.Security.Claims;
using Microsoft.AspNetCore.SignalR;

namespace NotificationService.Infrastructure.Hubs;

internal sealed class UserIdProvider : IUserIdProvider
{
    public string? GetUserId(HubConnectionContext connection)
    {
        return connection.User.Claims.FirstOrDefault(c => c.Type == ClaimTypes.NameIdentifier)?.Value;
    }
}