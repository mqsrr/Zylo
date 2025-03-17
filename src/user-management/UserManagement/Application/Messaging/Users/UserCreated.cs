using UserManagement.Application.Models;

namespace UserManagement.Application.Messaging.Users;

public sealed class UserCreated
{
    public required UserId Id { get; init; }

}