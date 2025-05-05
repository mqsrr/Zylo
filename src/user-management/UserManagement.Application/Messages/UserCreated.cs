using UserManagement.Domain.Users;

namespace UserManagement.Application.Messages;

public sealed class UserCreated
{
    public required UserId Id { get; init; }
}