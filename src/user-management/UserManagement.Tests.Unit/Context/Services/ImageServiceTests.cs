using System.Net;
using Amazon.S3;
using Amazon.S3.Model;
using FluentAssertions;
using Microsoft.AspNetCore.Http;
using Microsoft.Extensions.Options;
using NSubstitute;
using UserManagement.Application.Models;
using UserManagement.Application.Services;
using UserManagement.Application.Services.Abstractions;
using UserManagement.Application.Settings;

namespace UserManagement.Tests.Unit.Context.Services;

public sealed class ImageServiceFacts
{
    private readonly IAmazonS3 _s3Client;
    private readonly IOptions<S3Settings> _settings;
    private readonly TimeProvider _timeProvider;

    private readonly ImageService _sut;

    public ImageServiceFacts()
    {
        _s3Client = Substitute.For<IAmazonS3>();
        _timeProvider = Substitute.For<TimeProvider>();
        _timeProvider.GetUtcNow().Returns(DateTime.UtcNow);
        
        _settings = Substitute.For<IOptions<S3Settings>>();
        _settings.Value.Returns(new S3Settings
        {
            BucketName = "test-bucket",
            PresignedUrlExpire = 30
        });

        _sut = new ImageService(_s3Client, _settings, _timeProvider);
    }

    [Fact]
    public async Task GetImageAsync_ShouldReturnFileMetadata_WhenImageExists()
    {
        // Arrange
        var userId = UserId.NewId();
        const ImageCategory imageCategory = ImageCategory.Profile;
        var cancellationToken = CancellationToken.None;
        
        const string presignedUrl = "https://presigned-url.com/profile.jpg";
        string imageKey = $"profile_images/{userId}";

        var metadataResponse = new GetObjectMetadataResponse
        {
            HttpStatusCode = HttpStatusCode.OK,
            Headers =
            {
                ContentType = "image/jpeg"
            },
            Metadata =
            {
                ["file-name"] = "profile.jpg"
            }
        };

        _s3Client.GetObjectMetadataAsync(Arg.Is<GetObjectMetadataRequest>(r =>
            r.BucketName == _settings.Value.BucketName &&
            r.Key == imageKey), cancellationToken).Returns(metadataResponse);

        _s3Client.GetPreSignedURLAsync(Arg.Is<GetPreSignedUrlRequest>(r =>
            r.BucketName == _settings.Value.BucketName &&
            r.Key == imageKey &&
            r.Expires > DateTime.UtcNow)).Returns(presignedUrl);

        // Act
        var result = await _sut.GetImageAsync(userId, imageCategory, cancellationToken);

        // Assert
        result.Should().NotBeNull();
        
        result.AccessUrl.Url.Should().Be(presignedUrl);
        result.ContentType.Should().Be("image/jpeg");
        result.FileName.Should().Be("profile.jpg");

        await _s3Client.Received().GetObjectMetadataAsync(Arg.Is<GetObjectMetadataRequest>(r =>
            r.BucketName == _settings.Value.BucketName && r.Key == imageKey), cancellationToken);
        
        await _s3Client.Received().GetPreSignedURLAsync(Arg.Is<GetPreSignedUrlRequest>(r =>
            r.BucketName == _settings.Value.BucketName && r.Key == imageKey && r.Expires > DateTime.UtcNow));
    }

    [Fact]
    public async Task UploadImageAsync_ShouldReturnTrue_WhenUploadIsSuccessful()
    {
        // Arrange
        var userId = UserId.NewId();
        const ImageCategory imageCategory = ImageCategory.Profile;
        var formFile = Substitute.For<IFormFile>();
        var cancellationToken = CancellationToken.None;

        formFile.FileName.Returns("profile.jpg");
        formFile.ContentType.Returns("image/jpeg");
        formFile.OpenReadStream().Returns(Stream.Null);

        var putObjectResponse = new PutObjectResponse
        {
            HttpStatusCode = HttpStatusCode.OK
        };

        _s3Client.PutObjectAsync(Arg.Is<PutObjectRequest>(r =>
            r.BucketName == _settings.Value.BucketName &&
            r.Key == $"profile_images/{userId}" &&
            r.ContentType == formFile.ContentType &&
            r.Metadata["x-amz-meta-file-name"] == formFile.FileName), cancellationToken).Returns(putObjectResponse);

        // Act
        bool result = await _sut.UploadImageAsync(userId, formFile, imageCategory, cancellationToken);

        // Assert
        result.Should().BeTrue();
        await _s3Client.Received().PutObjectAsync(Arg.Is<PutObjectRequest>(r =>
            r.BucketName == _settings.Value.BucketName &&
            r.Key == $"profile_images/{userId}" &&
            r.ContentType == formFile.ContentType &&
            r.Metadata["x-amz-meta-file-name"] == formFile.FileName), cancellationToken);
    }

    [Fact]
    public async Task UploadImageAsync_ShouldReturnFalse_WhenUploadFails()
    {
        // Arrange
        var userId = UserId.NewId();
        const ImageCategory imageCategory = ImageCategory.Profile;
        var formFile = Substitute.For<IFormFile>();
        var cancellationToken = CancellationToken.None;

        formFile.FileName.Returns("profile.jpg");
        formFile.ContentType.Returns("image/jpeg");
        formFile.OpenReadStream().Returns(Stream.Null);

        var putObjectResponse = new PutObjectResponse
        {
            HttpStatusCode = HttpStatusCode.InternalServerError
        };

        _s3Client.PutObjectAsync(Arg.Is<PutObjectRequest>(r =>
            r.BucketName == _settings.Value.BucketName &&
            r.Key == $"profile_images/{userId}" &&
            r.ContentType == formFile.ContentType &&
            r.Metadata["x-amz-meta-file-name"] == formFile.FileName), cancellationToken).Returns(putObjectResponse);


        // Act
        bool result = await _sut.UploadImageAsync(userId, formFile, imageCategory, cancellationToken);

        // Assert
        result.Should().BeFalse();
        await _s3Client.Received().PutObjectAsync(Arg.Is<PutObjectRequest>(r =>
            r.BucketName == _settings.Value.BucketName &&
            r.Key == $"profile_images/{userId}" &&
            r.ContentType == formFile.ContentType &&
            r.Metadata["x-amz-meta-file-name"] == formFile.FileName), cancellationToken);
    }

    [Fact]
    public async Task DeleteAllImagesAsync_ShouldReturnTrue_WhenDeletionIsSuccessful()
    {
        // Arrange
        var userId = UserId.NewId();
        var cancellationToken = CancellationToken.None;

        var deleteObjectsResponse = new DeleteObjectsResponse
        {
            HttpStatusCode = HttpStatusCode.NoContent
        };

        _s3Client.DeleteObjectsAsync(Arg.Is<DeleteObjectsRequest>(request =>
                request.BucketName == _settings.Value.BucketName &&
                request.Objects.Count == 2 &&
                request.Objects.Any(o => o.Key == $"profile_images/{userId}") &&
                request.Objects.Any(o => o.Key == $"background_images/{userId}")), cancellationToken)
            .Returns(deleteObjectsResponse);

        // Act
        bool result = await _sut.DeleteAllImagesAsync(userId, cancellationToken);

        // Assert
        result.Should().BeTrue();
        await _s3Client.Received().DeleteObjectsAsync(Arg.Is<DeleteObjectsRequest>(request =>
            request.BucketName == _settings.Value.BucketName &&
            request.Objects.Count == 2 &&
            request.Objects.Any(o => o.Key == $"profile_images/{userId}") &&
            request.Objects.Any(o => o.Key == $"background_images/{userId}")), cancellationToken);
    }

    [Fact]
    public async Task DeleteAllImagesAsync_ShouldReturnFalse_WhenDeletionFails()
    {
        // Arrange
        var userId = UserId.NewId();
        var cancellationToken = CancellationToken.None;

        var deleteObjectsResponse = new DeleteObjectsResponse
        {
            HttpStatusCode = HttpStatusCode.InternalServerError
        };

        _s3Client.DeleteObjectsAsync(Arg.Is<DeleteObjectsRequest>(request =>
            request.BucketName == _settings.Value.BucketName &&
            request.Objects.Any(o => o.Key == $"profile_images/{userId}") &&
            request.Objects.Any(o => o.Key == $"background_images/{userId}")), cancellationToken).Returns(deleteObjectsResponse);


        // Act
        bool result = await _sut.DeleteAllImagesAsync(userId, cancellationToken);

        // Assert
        result.Should().BeFalse();
        await _s3Client.Received().DeleteObjectsAsync(Arg.Is<DeleteObjectsRequest>(request =>
            request.BucketName == _settings.Value.BucketName &&
            request.Objects.Any(o => o.Key == $"profile_images/{userId}") &&
            request.Objects.Any(o => o.Key == $"background_images/{userId}")), cancellationToken);
    }
}