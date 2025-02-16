using UserManagement.Application.Models;

namespace UserManagement.Application.Messaging.Users;

public sealed class UserDeleted
{
     public required UserId Id { get; init; }
}