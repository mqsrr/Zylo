
namespace ApiGateway.Models;

public sealed class PostInteractionResponse
{
    public required Ulid PostId { get; init; }
    
    public required IEnumerable<Reply> Replies { get; init; }
    
    public required int Likes { get; init; }

    public required int Views { get; init; }
    
    public bool? UserInteracted { get; init; }
}

public sealed class Reply
{
    public required Ulid Id { get; set; }

    public required UserSummary User { get; init; }

    public required Ulid ReplyToId { get; init; }

    public required string Content { get; init; }

    public required DateTime CreatedAt { get; init; }

    public required IEnumerable<Reply> NestedReplies { get; init; }

    public required int Likes { get; init; }

    public required int Views { get; init; }
    
    public bool? UserInteracted { get; init; }
}

