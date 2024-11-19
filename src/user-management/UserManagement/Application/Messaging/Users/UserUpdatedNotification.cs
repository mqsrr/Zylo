using Mediator;
using UserManagement.Application.Models;

namespace UserManagement.Application.Messaging.Users;

public sealed class UserUpdatedNotification : INotification
{
    public required UserId Id { get; init; }
    
    
    public required string Name { get; init; }
    
    public required string Bio { get; init; }
    
    public required string Location { get; init; }
}