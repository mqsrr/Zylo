using UserManagement.Application.Models;

namespace UserManagement.Application.Contracts.Requests.Users;

public sealed class UpdateUserRequest
{
    public UserId Id { get; internal set; }
    
    public required IFormFile ProfileImage { get; init; }

    public required IFormFile BackgroundImage { get; init; }

    public required string Name { get; init; }
    
    public required string Bio { get; init; }

    public required string Location { get; init; }

    public required DateOnly BirthDate { get; init; }

}