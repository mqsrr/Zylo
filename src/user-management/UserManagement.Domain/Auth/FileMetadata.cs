namespace UserManagement.Domain.Auth;

public sealed class FileMetadata
{
    public required PresignedUrl AccessUrl { get; init; }

    public required string ContentType { get; init; }

    public required string FileName { get; init; }
}

public sealed class PresignedUrl
{
    public required string Url { get; init; }

    public required DateTime ExpiresIn { get; init; }
}