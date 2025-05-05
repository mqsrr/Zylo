using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.SignalR;
using NotificationService.Application.Abstractions;

namespace NotificationService.Infrastructure.Hubs;

[Authorize]
public sealed class NotificationHub : Hub<INotificationHub>
{
    public async Task JoinFriendGroup(string friendUserId)
    {
        await Groups.AddToGroupAsync(Context.ConnectionId, $"{friendUserId}-friends");
    }

    public async Task LeaveFriendGroup(string friendUserId)
    {
        await Groups.RemoveFromGroupAsync(Context.ConnectionId, $"{friendUserId}-friends");
    }

    public async Task JoinFollowerGroup(string followerUserId)
    {
        await Groups.AddToGroupAsync(Context.ConnectionId, $"{followerUserId}-followers");
    }

    public async Task LeaveFollowerGroup(string followerUserId)
    {
        await Groups.RemoveFromGroupAsync(Context.ConnectionId, $"{followerUserId}-followers");
    }

    public override async Task OnConnectedAsync()
    {
        string userId = Context.UserIdentifier!;
        
        await Groups.AddToGroupAsync(Context.ConnectionId, userId);
        await base.OnConnectedAsync();
    }
}