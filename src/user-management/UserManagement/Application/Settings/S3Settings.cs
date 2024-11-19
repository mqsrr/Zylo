namespace UserManagement.Application.Settings;

public sealed class S3Settings
{
    public const string SectionName = "S3";
    
    public required string BucketName { get; init; }

    public required int PresignedUrlExpire { get; init; }
}