namespace UserManagement.Application.Messaging.Users;

public interface UserCreated
{
    Ulid Id { get; }
    
    string Username { get; }
    
    string Name { get; }
}