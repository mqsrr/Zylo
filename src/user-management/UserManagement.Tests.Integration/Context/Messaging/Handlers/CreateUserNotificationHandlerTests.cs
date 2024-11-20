using System.Net;
using Amazon.S3.Model;
using AutoFixture;
using Dapper;
using FluentAssertions;
using MassTransit;
using MassTransit.Testing;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Options;
using Npgsql;
using NSubstitute;
using NSubstitute.ClearExtensions;
using UserManagement.Application.Contracts.Requests.Auth;
using UserManagement.Application.Factories.Abstractions;
using UserManagement.Application.Helpers;
using UserManagement.Application.Mappers;
using UserManagement.Application.Messaging.Users;
using UserManagement.Application.Messaging.Users.Handlers;
using UserManagement.Application.Models;
using UserManagement.Application.Repositories.Abstractions;
using UserManagement.Application.Settings;
using UserManagement.Tests.Integration.Fixtures;
using UserManagement.Tests.Integration.Fixtures.Customizations;

namespace UserManagement.Tests.Integration.Context.Messaging.Handlers;

[Collection(nameof(UserManagementApiCollection))]
public sealed class CreateUserNotificationHandlerTests : IAsyncDisposable
{
    private readonly UserManagementApiFactory _factory;
    private readonly IDbConnectionFactory _dbConnectionFactory;
    private readonly ITestHarness _testHarness;
    private readonly S3Settings _s3Settings;
    private readonly Fixture _fixture;
    private readonly AsyncServiceScope _serviceScope;

    private readonly CreateUserNotificationHandler _sut;

    public CreateUserNotificationHandlerTests(UserManagementApiFactory factory)
    {
        _factory = factory;
        _fixture = new Fixture();
        _fixture.Customize(new RegisterRequestCustomization());

        _serviceScope = _factory.Services.CreateAsyncScope();
        _s3Settings = _serviceScope.ServiceProvider.GetRequiredService<IOptions<S3Settings>>().Value;
        _dbConnectionFactory = _serviceScope.ServiceProvider.GetRequiredService<IDbConnectionFactory>();
        _testHarness = _serviceScope.ServiceProvider.GetTestHarness();

        var publishEndpoint = _serviceScope.ServiceProvider.GetRequiredService<IPublishEndpoint>();
        var userRepository = _serviceScope.ServiceProvider.GetRequiredService<IUserRepository>();

        _sut = new CreateUserNotificationHandler(userRepository, publishEndpoint);
    }

    [Fact]
    public async Task Handle_ShouldPublishUserCreatedMessage_WhenUserIsCreated()
    {
        // Arrange
        _factory.S3.ClearSubstitute();
        var registerRequest = _fixture.Create<RegisterRequest>();
        var createUserNotification = new CreateUserNotification
        {
            Request = registerRequest
        };

        var expectedUser = registerRequest.ToUser();
        var cancellationToken = CancellationToken.None;

        string imageKey = $"profile_images/{registerRequest.Id}";
        string backgroundImageKey = $"background_images/{registerRequest.Id}";

        var objectMetadataResponse = new PutObjectResponse
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
            .Returns(objectMetadataResponse);

        // Act
        await _sut.Handle(createUserNotification, cancellationToken);

        // Assert

        await using var connection = await _dbConnectionFactory.CreateAsync(cancellationToken);
        var createdUser = await connection.QueryFirstOrDefaultAsync<User>(SqlQueries.Users.GetById, new {registerRequest.Id});
        createdUser.Should().BeEquivalentTo(expectedUser);

        await _factory.S3.Received(2).PutObjectAsync(Arg.Is<PutObjectRequest>(r =>
                r.BucketName == _s3Settings.BucketName &&
                (r.Key == imageKey || r.Key == backgroundImageKey) &&
                (r.ContentType == registerRequest.ProfileImage.ContentType || r.ContentType == registerRequest.BackgroundImage.ContentType) &&
                (r.Metadata["x-amz-meta-file-name"] == registerRequest.ProfileImage.FileName ||
                 r.Metadata["x-amz-meta-file-name"] == registerRequest.BackgroundImage.FileName)),
            cancellationToken);

        var publishedMessage = _testHarness.Published.Select<UserCreated>(cancellationToken)
            .FirstOrDefault(publishedMessage => publishedMessage.Context.Message.Id == registerRequest.Id.Value);

        publishedMessage.Should().NotBeNull();
        publishedMessage!.Context.Message.Should().BeEquivalentTo(expectedUser, options => options.ExcludingMissingMembers());
        publishedMessage.Context.RoutingKey().Should().BeEquivalentTo("user.created");
    }

    [Fact]
    public async Task Handle_ShouldNotPublishUserCreatedMessage_WhenUserWasNotCreated()
    {
        // Arrange
        _factory.S3.ClearSubstitute();
        var registerRequest = _fixture.Create<RegisterRequest>();
        var expectedUser = registerRequest.ToUser();
        var cancellationToken = CancellationToken.None;
        var createUserNotification = new CreateUserNotification
        {
            Request = registerRequest
        };

        await using var connection = await _dbConnectionFactory.CreateAsync(cancellationToken);
        await connection.ExecuteAsync(SqlQueries.Users.Create, expectedUser);

        // Act
        var handleFunc = async () => await _sut.Handle(createUserNotification, cancellationToken);

        // Assert
        await handleFunc.Should().NotThrowAsync<NpgsqlException>();
        _testHarness.Published.Select<UserCreated>(cancellationToken)
            .FirstOrDefault(publishedMessage => publishedMessage.Context.Message.Id == registerRequest.Id.Value)
            .Should()
            .BeNull();
    }

    public async ValueTask DisposeAsync()
    {
        await _serviceScope.DisposeAsync();
    }
}