using System.Net;
using Amazon.S3.Model;
using AutoFixture;
using Dapper;
using FluentAssertions;
using MassTransit;
using MassTransit.Testing;
using Mediator;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Options;
using Npgsql;
using NSubstitute;
using NSubstitute.ClearExtensions;
using UserManagement.Application.Contracts.Requests.Auth;
using UserManagement.Application.Contracts.Requests.Users;
using UserManagement.Application.Factories.Abstractions;
using UserManagement.Application.Helpers;
using UserManagement.Application.Mappers;
using UserManagement.Application.Messaging.Users;
using UserManagement.Application.Models;
using UserManagement.Application.Repositories;
using UserManagement.Application.Services.Abstractions;
using UserManagement.Application.Settings;
using UserManagement.Tests.Integration.Fixtures;
using UserManagement.Tests.Integration.Fixtures.Customizations;

namespace UserManagement.Tests.Integration.Context.Repositories;

[Collection(nameof(UserManagementApiCollection))]
public sealed class UserRepositoryTests : IAsyncDisposable
{
    private readonly UserManagementApiFactory _factory;
    private readonly ITestHarness _testHarness;
    private readonly Fixture _fixture;
    private readonly S3Settings _s3Settings;
    private readonly IDbConnectionFactory _dbConnectionFactory;
    private readonly AsyncServiceScope _serviceScope;

    private readonly UserRepository _sut;

    public UserRepositoryTests(UserManagementApiFactory factory)
    {
        _factory = factory;
        _fixture = new Fixture();
        _fixture.Customize(new UserCustomization())
            .Customize(new RegisterRequestCustomization());

        _serviceScope = _factory.Services.CreateAsyncScope();
        var imageService = _serviceScope.ServiceProvider.GetRequiredService<IImageService>();
        _testHarness = _serviceScope.ServiceProvider.GetTestHarness();
        _dbConnectionFactory = _serviceScope.ServiceProvider.GetRequiredService<IDbConnectionFactory>();
        _s3Settings = _serviceScope.ServiceProvider.GetRequiredService<IOptions<S3Settings>>().Value;

        var publisher = _serviceScope.ServiceProvider.GetRequiredService<IPublisher>();
        _sut = new UserRepository(_dbConnectionFactory, publisher, imageService);
    }

    [Fact]
    public async Task GetByIdAsync_ShouldReturnUser_WhenUserExists()
    {
        // Arrange
        _factory.S3.ClearSubstitute();

        var user = _fixture.Create<User>();
        var cancellationToken = CancellationToken.None;

        await using var connection = await _dbConnectionFactory.CreateAsync(cancellationToken);
        await connection.ExecuteAsync(SqlQueries.Users.Create, user);

        const string presignedUrl = "profile_image/url";
        string imageKey = $"profile_images/{user.Id}";
        string backgroundImageKey = $"background_images/{user.Id}";

        var getObjectMetadataResponse = new GetObjectMetadataResponse
        {
            HttpStatusCode = HttpStatusCode.OK,
            Metadata =
            {
                ["Content-Type"] = "image/jpeg",
                ["file-name"] = "profile.jpg"
            }
        };

        _factory.S3.GetObjectMetadataAsync(Arg.Is<GetObjectMetadataRequest>(r =>
                    r.BucketName == _s3Settings.BucketName &&
                    (r.Key == imageKey || r.Key == backgroundImageKey)),
                cancellationToken)
            .Returns(getObjectMetadataResponse);

        _factory.S3.GetPreSignedURLAsync(Arg.Is<GetPreSignedUrlRequest>(r =>
                r.BucketName == _s3Settings.BucketName &&
                (r.Key == imageKey || r.Key == backgroundImageKey)))
            .Returns(presignedUrl);

        // Act
        var result = await _sut.GetByIdAsync(user.Id, cancellationToken);

        // Assert
        result.Should().NotBeNull();
        result.Should().BeEquivalentTo(user, options => options.Excluding(u => u.ProfileImage).Excluding(u => u.BackgroundImage));

        await _factory.S3.Received(2).GetObjectMetadataAsync(Arg.Is<GetObjectMetadataRequest>(r =>
                r.BucketName == _s3Settings.BucketName &&
                (r.Key == imageKey || r.Key == backgroundImageKey)),
            cancellationToken);

        await _factory.S3.Received(2).GetPreSignedURLAsync(Arg.Is<GetPreSignedUrlRequest>(r =>
            r.BucketName == _s3Settings.BucketName &&
            (r.Key == imageKey || r.Key == backgroundImageKey)));
    }

    [Fact]
    public async Task GetByIdAsync_ShouldReturnNull_WhenUserDoesNotExist()
    {
        // Arrange
        _factory.S3.ClearSubstitute();
        var userId = UserId.NewId();
        var cancellationToken = CancellationToken.None;

        // Act
        var result = await _sut.GetByIdAsync(userId, cancellationToken);

        // Assert
        result.Should().BeNull();

        await _factory.S3.DidNotReceiveWithAnyArgs().GetObjectMetadataAsync(default, cancellationToken);
        await _factory.S3.DidNotReceiveWithAnyArgs().GetPreSignedURLAsync(default);
    }

    [Fact]
    public async Task CreateAsync_ShouldReturnTrue_WhenUserIsCreated()
    {
        // Arrange
        _factory.S3.ClearSubstitute();
        var registerRequest = _fixture.Create<RegisterRequest>();

        var user = registerRequest.ToUser();
        var cancellationToken = CancellationToken.None;

        string imageKey = $"profile_images/{user.Id}";
        string backgroundImageKey = $"background_images/{user.Id}";

        var putObjectResponse = new PutObjectResponse
        {
            HttpStatusCode = HttpStatusCode.OK
        };

        _factory.S3.PutObjectAsync(Arg.Is<PutObjectRequest>(r =>
                    r.BucketName == _s3Settings.BucketName &&
                    (r.Key == imageKey || r.Key == backgroundImageKey) &&
                    (r.ContentType == registerRequest.ProfileImage.ContentType || r.ContentType == registerRequest.BackgroundImage.ContentType) &&
                    (r.Metadata["x-amz-meta-file-name"] == registerRequest.ProfileImage.FileName ||
                     r.Metadata["x-amz-meta-file-name"] == registerRequest.BackgroundImage.FileName)),
                cancellationToken)
            .Returns(putObjectResponse);


        // Act
        bool result = await _sut.CreateAsync(user, registerRequest.ProfileImage, registerRequest.BackgroundImage, cancellationToken);

        // Assert
        result.Should().BeTrue();
        
        await using var connection = await _dbConnectionFactory.CreateAsync(cancellationToken);
        var createdUser = await connection.QueryFirstOrDefaultAsync<User>(SqlQueries.Users.GetById, new {user.Id});
        createdUser.Should().BeEquivalentTo(user);
        
        await _factory.S3.Received(2).PutObjectAsync(Arg.Is<PutObjectRequest>(r =>
                r.BucketName == _s3Settings.BucketName &&
                (r.Key == imageKey || r.Key == backgroundImageKey) &&
                (r.ContentType == registerRequest.ProfileImage.ContentType || r.ContentType == registerRequest.BackgroundImage.ContentType) &&
                (r.Metadata["x-amz-meta-file-name"] == registerRequest.ProfileImage.FileName ||
                 r.Metadata["x-amz-meta-file-name"] == registerRequest.BackgroundImage.FileName)),
            cancellationToken);
    }
    
    [Fact]
    public async Task CreateAsync_ShouldThrowException_WhenUserAlreadyExists()
    {
        // Arrange
        _factory.S3.ClearSubstitute();
        var registerRequest = _fixture.Create<RegisterRequest>();

        var user = registerRequest.ToUser();
        var cancellationToken = CancellationToken.None;

        await using var connection = await _dbConnectionFactory.CreateAsync(cancellationToken);
        await connection.ExecuteAsync(SqlQueries.Users.Create, user);

        // Act
        var createFunc = async () => await _sut.CreateAsync(user, registerRequest.ProfileImage, registerRequest.BackgroundImage, cancellationToken);

        // Assert
        await createFunc.Should().ThrowAsync<NpgsqlException>();
    }

    [Fact]
    public async Task UpdateAsync_ShouldReturnTrue_WhenUserIsUpdated()
    {
        // Arrange
        _factory.S3.ClearSubstitute();

        var user = _fixture.Create<User>();
        var updateUserRequest = _fixture.Create<UpdateUserRequest>();
        updateUserRequest.Id = user.Id;
        
        var cancellationToken = CancellationToken.None;

        await using var connection = await _dbConnectionFactory.CreateAsync(cancellationToken);
        await connection.ExecuteAsync(SqlQueries.Users.Create, user);

        string imageKey = $"profile_images/{user.Id}";
        string backgroundImageKey = $"background_images/{user.Id}";

        var putObjectResponse = new PutObjectResponse
        {
            HttpStatusCode = HttpStatusCode.OK
        };

        _factory.S3.PutObjectAsync(Arg.Is<PutObjectRequest>(r =>
                    r.BucketName == _s3Settings.BucketName &&
                    (r.Key == imageKey || r.Key == backgroundImageKey) &&
                    (r.ContentType == updateUserRequest.ProfileImage.ContentType || r.ContentType == updateUserRequest.BackgroundImage.ContentType) &&
                    (r.Metadata["x-amz-meta-file-name"] == updateUserRequest.ProfileImage.FileName ||
                     r.Metadata["x-amz-meta-file-name"] == updateUserRequest.BackgroundImage.FileName)),
                cancellationToken)
            .Returns(putObjectResponse);

        // Act
        bool result = await _sut.UpdateAsync(updateUserRequest, updateUserRequest.ProfileImage, updateUserRequest.BackgroundImage, cancellationToken);

        // Assert
        result.Should().BeTrue();

        var updatedUser = await connection.QueryFirstOrDefaultAsync<User>(SqlQueries.Users.GetById, new {user.Id});
        updatedUser.Should().BeEquivalentTo(updateUserRequest, options => options.Excluding(u => u.ProfileImage).Excluding(u => u.BackgroundImage).ExcludingMissingMembers());
        
        await _factory.S3.Received(2).PutObjectAsync(Arg.Is<PutObjectRequest>(r =>
                r.BucketName == _s3Settings.BucketName &&
                (r.Key == imageKey || r.Key == backgroundImageKey) &&
                (r.ContentType == updateUserRequest.ProfileImage.ContentType || r.ContentType == updateUserRequest.BackgroundImage.ContentType) &&
                (r.Metadata["x-amz-meta-file-name"] == updateUserRequest.ProfileImage.FileName ||
                 r.Metadata["x-amz-meta-file-name"] == updateUserRequest.BackgroundImage.FileName)),
            cancellationToken);
        
        var publishedMessage = _testHarness.Published.Select<UserUpdated>(cancellationToken)
            .FirstOrDefault(publishedMessage => publishedMessage.Context.Message.Id == updateUserRequest.Id.Value);

        publishedMessage.Should().NotBeNull();
        publishedMessage!.Context.Message.Should().BeEquivalentTo(updatedUser, options => options.Excluding(u => u!.ProfileImage).Excluding(u => u!.BackgroundImage).ExcludingMissingMembers());
        publishedMessage.Context.RoutingKey().Should().BeEquivalentTo("user.updated");
    }
    
    [Fact]
    public async Task UpdateAsync_ShouldReturnFalse_WhenUserWasNotFound()
    {
        // Arrange
        _factory.S3.ClearSubstitute();

        var updateUserRequest = _fixture.Create<UpdateUserRequest>();
        var cancellationToken = CancellationToken.None;

        await using var connection = await _dbConnectionFactory.CreateAsync(cancellationToken);

        // Act
        bool result = await _sut.UpdateAsync(updateUserRequest, updateUserRequest.ProfileImage, updateUserRequest.BackgroundImage, cancellationToken);

        // Assert
        result.Should().BeFalse();
        await _factory.S3.DidNotReceiveWithAnyArgs().PutObjectAsync(default, cancellationToken);
        
        _testHarness.Published.Select<UserUpdated>(cancellationToken)
            .FirstOrDefault(publishedMessage => publishedMessage.Context.Message.Id == updateUserRequest.Id.Value)
            .Should()
            .BeNull();
    }

    [Fact]
    public async Task DeleteByIdAsync_ShouldReturnTrue_WhenUserIsDeleted()
    {
        // Arrange
        _factory.S3.ClearSubstitute();

        var registerRequest = _fixture.Create<RegisterRequest>();
        var identity = registerRequest.ToIdentity();
        
        var user = registerRequest.ToUser();
        var cancellationToken = CancellationToken.None;

        await using var connection = await _dbConnectionFactory.CreateAsync(cancellationToken);
        await connection.ExecuteAsync(SqlQueries.Authentication.Register, identity);
        await connection.ExecuteAsync(SqlQueries.Users.Create, user);

        string imageKey = $"profile_images/{user.Id}";
        string backgroundImageKey = $"background_images/{user.Id}";

        var objectMetadataResponse = new DeleteObjectsResponse
        {
            HttpStatusCode = HttpStatusCode.OK
        };

        _factory.S3.DeleteObjectsAsync(Arg.Is<DeleteObjectsRequest>(request =>
                    request.BucketName == _s3Settings.BucketName &&
                    request.Objects.Count == 2 &&
                    request.Objects.Any(o => o.Key == imageKey) &&
                    request.Objects.Any(o => o.Key == backgroundImageKey)), cancellationToken)
            .Returns(objectMetadataResponse);

        // Act
        bool result = await _sut.DeleteByIdAsync(user.Id, cancellationToken);

        // Assert
        result.Should().BeTrue();
        var deletedUser = await connection.QueryFirstOrDefaultAsync<User>(SqlQueries.Users.GetById, new {user.Id});
        var deletedIdentity = await connection.QueryFirstOrDefaultAsync<User>(SqlQueries.Users.GetById, new {user.Id});
        
        deletedUser.Should().BeNull();
        deletedIdentity.Should().BeNull();
        
        await _factory.S3.Received().DeleteObjectsAsync(Arg.Is<DeleteObjectsRequest>(request =>
            request.BucketName == _s3Settings.BucketName &&
            request.Objects.Count == 2 &&
            request.Objects.Any(o => o.Key == imageKey) &&
            request.Objects.Any(o => o.Key == backgroundImageKey)), cancellationToken);
            
        var publishedMessage = _testHarness.Published.Select<UserDeleted>(cancellationToken)
            .FirstOrDefault(publishedMessage => publishedMessage.Context.Message.Id == user.Id.Value);

        publishedMessage.Should().NotBeNull();
        publishedMessage!.Context.RoutingKey().Should().BeEquivalentTo("user.deleted");
    }
    
    [Fact]
    public async Task DeleteByIdAsync_ShouldReturnFalse_WhenUserDoesNotExist()
    {
        // Arrange
        _factory.S3.ClearSubstitute();

        var userId = UserId.NewId();
        var cancellationToken = CancellationToken.None;

        
        // Act
        bool result = await _sut.DeleteByIdAsync(userId, cancellationToken);

        // Assert
        result.Should().BeFalse();
        _testHarness.Published.Select<UserDeleted>(cancellationToken)
            .FirstOrDefault(publishedMessage => publishedMessage.Context.Message.Id == userId.Value)
            .Should()
            .BeNull();
    }
    
    public async ValueTask DisposeAsync()
    {
        await _serviceScope.DisposeAsync();
    }
}