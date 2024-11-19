using Mediator;
using UserManagement.Application.Models;

namespace UserManagement.Application.Messaging.Users;

public sealed record UserDeletedNotification : INotification
{
    public required UserId Id { get; init; }
}