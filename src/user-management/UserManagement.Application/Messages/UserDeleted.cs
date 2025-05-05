using UserManagement.Domain.Users;

namespace UserManagement.Application.Messages;

public sealed class UserDeleted
{
     public required UserId Id { get; init; }
}