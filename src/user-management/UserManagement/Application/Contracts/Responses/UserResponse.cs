using UserManagement.Application.Models;

namespace UserManagement.Application.Contracts.Responses;

internal sealed class UserResponse
{
    public required Ulid Id { get; init; }

    public required FileMetadata ProfileImage { get; init; }

    public required FileMetadata BackgroundImage { get; init; }

    public required string Name { get; init; }
    
    public required string Username { get; init; }

    public string? Bio { get; init; }

    public string? Location { get; init; }

    public required DateOnly BirthDate { get; init; }

}