namespace ApiGateway.Models;

public sealed class FileMetadata
{
    public required string FileName { get; init; }

    public required string ContentType { get; init; }

    public required string Url { get; init; }
}