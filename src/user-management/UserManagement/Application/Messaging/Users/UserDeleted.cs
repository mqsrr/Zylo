using UserManagement.Application.Models;

namespace UserManagement.Application.Messaging.Users;

public interface UserDeleted
{
     Ulid Id { get; }
}