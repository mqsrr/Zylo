using Mediator;
using UserManagement.Application.Contracts.Requests.Auth;

namespace UserManagement.Application.Messaging.Users;

public sealed record CreateUserNotification : INotification
{
    public required RegisterRequest Request { get; init; }
}