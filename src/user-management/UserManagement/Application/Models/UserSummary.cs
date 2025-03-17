namespace UserManagement.Application.Models;

public sealed class UserSummary
{
    public required UserId Id { get; init; }
    
    public required string Name { get; init; }
    
    public FileMetadata? ProfileImage { get; set; }
}