
namespace UserManagement.Application.Messaging.Users;

public interface UserUpdated
{
    Ulid Id { get; }
    
    string Name { get; }
    
    string Bio { get; }
    
    string Location { get; }
}