

namespace ApiGateway.Models;

public sealed class UserProfile
{
    public required Ulid Id { get; init; }

    public required FileMetadata ProfileImage { get; init; }

    public required FileMetadata BackgroundImage { get; init; }

    public required string Name { get; init; }
    
    public required string Username { get; init; }

    public required DateOnly BirthDate { get; init; }

    public UserRelationships? Relationships { get; set; }
    
    public string? Bio { get; init; }

    public string? Location { get; init; }

    public PaginatedResponse<Post>? Posts { get; set; }
    
}

public sealed class UserSummary
{
    public required Ulid Id { get; init; }

    public required FileMetadata ProfileImage { get; init; }
    
    public required string Name { get; init; }
    
    public required string Username { get; init; }
    
    public string? Bio { get; init; }

    public string? Location { get; init; }
}

public sealed class UserRelationships
{
    public IEnumerable<UserSummary>? Followers { get; init; }

    public IEnumerable<UserSummary>? FollowedPeople { get; init; }

    public IEnumerable<UserSummary>? BlockedPeople { get; init; }

    public IEnumerable<UserSummary>? Friends { get; init; }

    public IEnumerable<UserSummary>? SentFriendRequests { get; init; }

    public IEnumerable<UserSummary>? ReceivedFriendRequests { get; init; }
}