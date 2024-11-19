namespace ApiGateway.Models;

public class Post
{
    public required Ulid Id { get; init; }

    public required UserSummary User { get; init; }
    
    public required string Text { get; init; }
    
    public required DateTime CreatedAt { get; init; }

    public required DateTime UpdatedAt { get; init; }

    public IEnumerable<FileMetadata>? FilesMetadata { get; init; }

    public IEnumerable<Reply>? Replies { get; set; }
    
    public bool? UserInteracted { get; set; }

    public int Likes { get; set; }

    public int Views { get; set; }
}