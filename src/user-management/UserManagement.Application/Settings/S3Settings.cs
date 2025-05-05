namespace UserManagement.Application.Settings;

public sealed class S3Settings() : BaseSettings("S3")
{
    public required string BucketName { get; init; }

    public required int PresignedUrlExpire { get; init; }
}