using System.Net;
using Amazon.S3;
using Amazon.S3.Model;
using Microsoft.Extensions.Options;
using UserManagement.Application.Models;
using UserManagement.Application.Services.Abstractions;
using UserManagement.Application.Settings;

namespace UserManagement.Application.Services;

internal sealed class ImageService : IImageService
{
    private readonly IAmazonS3 _s3Client;
    private readonly S3Settings _settings;
    private readonly TimeProvider _timeProvider;

    public ImageService(IAmazonS3 s3Client, IOptions<S3Settings> settings, TimeProvider timeProvider)
    {
        _s3Client = s3Client;
        _settings = settings.Value;
        _timeProvider = timeProvider;
    }

    public async Task<FileMetadata> GetImageAsync(UserId id, ImageCategory category, CancellationToken cancellationToken)
    {
        string imageKey = GetImageKey(id, category);
        var expiresAt = _timeProvider.GetUtcNow().AddMinutes(_settings.PresignedUrlExpire).UtcDateTime;
        
        var request = new GetPreSignedUrlRequest
        {
            BucketName = _settings.BucketName,
            Key = imageKey,
            Expires = expiresAt
        };

        var headRequest = new GetObjectMetadataRequest
        {
            BucketName = _settings.BucketName,
            Key = imageKey
        };
        
        string url = await _s3Client.GetPreSignedURLAsync(request);
        var metadata = await _s3Client.GetObjectMetadataAsync(headRequest, cancellationToken);
        return new FileMetadata
        {
            AccessUrl = new PresignedUrl
            {
                Url = url,
                ExpiresIn = expiresAt
            },
            ContentType = metadata.Headers.ContentType,
            FileName = metadata.Metadata["file-name"]
        };
    }

    public async Task<bool> UploadImageAsync(UserId id, IFormFile file, ImageCategory category, CancellationToken cancellation)
    {
        string imageKey = GetImageKey(id, category);
        var uploadRequest = new PutObjectRequest
        {
            BucketName = _settings.BucketName,
            Key = imageKey,
            ContentType = file.ContentType,
            InputStream = file.OpenReadStream(),
            Metadata =
            {
                ["x-amz-meta-file-name"] = file.FileName,
                ["x-amz-meta-extension"] = Path.GetExtension(file.FileName),
            }
        };
        
        var uploadResponse = await _s3Client.PutObjectAsync(uploadRequest, cancellation);
        return uploadResponse.HttpStatusCode == HttpStatusCode.OK;
    }

    public async Task<bool> DeleteAllImagesAsync(UserId id, CancellationToken cancellationToken)
    {
        var request = new DeleteObjectsRequest
        {
            BucketName = _settings.BucketName,
            Objects =
            [
                new KeyVersion {Key = $"profile_images/{id}"},
                new KeyVersion {Key = $"background_images/{id}"},
            ],
            Quiet = true
        };

        var response = await _s3Client.DeleteObjectsAsync(request, cancellationToken);
        return response.HttpStatusCode == HttpStatusCode.NoContent;
    }

    private static string GetImageKey(UserId id, ImageCategory category)
    {
        return category == ImageCategory.Profile
            ? $"profile_images/{id}"
            : $"background_images/{id}";
    }
}