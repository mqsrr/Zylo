using AutoFixture;
using Dapper;
using FluentAssertions;
using MassTransit;
using MassTransit.Testing;
using Microsoft.Extensions.DependencyInjection;
using UserManagement.Application.Contracts.Requests.Auth;
using UserManagement.Application.Factories.Abstractions;
using UserManagement.Application.Helpers;
using UserManagement.Application.Mappers;
using UserManagement.Application.Messaging.Users;
using UserManagement.Application.Messaging.Users.Handlers;
using UserManagement.Application.Models;
using UserManagement.Application.Services.Abstractions;
using UserManagement.Tests.Integration.Fixtures;
using UserManagement.Tests.Integration.Fixtures.Customizations;

namespace UserManagement.Tests.Integration.Context.Messaging.Handlers;

[Collection(nameof(UserManagementApiCollection))]
public sealed class UserDeletedNotificationHandlerTests : IAsyncDisposable
{
    private readonly ITestHarness _testHarness;
    private readonly IDbConnectionFactory _dbConnectionFactory;
    private readonly Fixture _fixture;
    private readonly AsyncServiceScope _serviceScope;

    private readonly UserDeletedNotificationHandler _sut;

    public UserDeletedNotificationHandlerTests(UserManagementApiFactory factory)
    {
        _fixture = new Fixture();
        _fixture.Customize(new RegisterRequestCustomization());

        _serviceScope = factory.Services.CreateAsyncScope();
        _dbConnectionFactory = _serviceScope.ServiceProvider.GetRequiredService<IDbConnectionFactory>();
        _testHarness = _serviceScope.ServiceProvider.GetTestHarness();

        var publishEndpoint = _serviceScope.ServiceProvider.GetRequiredService<IPublishEndpoint>();
        var authService = _serviceScope.ServiceProvider.GetRequiredService<IAuthService>();
        _sut = new UserDeletedNotificationHandler(authService, publishEndpoint);
    }
    
    [Fact]
    public async Task Handle_ShouldPublishUserDeletedMessage_WhenUserIsDeleted()
    {
        // Arrange
        var cancellationToken = CancellationToken.None;
        var identity = _fixture.Create<RegisterRequest>().ToIdentity();
        var userDeletedNotification = _fixture.Build<UserDeletedNotification>()
            .With(n => n.Id, UserId.Parse(identity.Id.Value.ToString()))
            .Create();
        
        await using var connection = await _dbConnectionFactory.CreateAsync(cancellationToken);
        await connection.ExecuteAsync(SqlQueries.Authentication.Register, identity);

        // Act
        await _sut.Handle(userDeletedNotification, cancellationToken);

        // Assert
        var publishedMessage = _testHarness.Published.Select<UserDeleted>(cancellationToken)
            .FirstOrDefault(publishedMessage => publishedMessage.Context.Message.Id == identity.Id.Value);

        publishedMessage.Should().NotBeNull();
        publishedMessage!.Context.Message.Id.Should().BeEquivalentTo(identity.Id.Value);
        publishedMessage.Context.RoutingKey().Should().BeEquivalentTo("user.deleted");
    }
    
    [Fact]
    public async Task Handle_ShouldNotPublishUserDeletedMessage_WhenUserWasNotDeleted()
    {
        // Arrange
        var cancellationToken = CancellationToken.None;
        var userDeletedNotification = new UserDeletedNotification
        {
            Id = UserId.NewId()
        };

        // Act
        await _sut.Handle(userDeletedNotification, cancellationToken);

        // Assert
        _testHarness.Published.Select<UserDeleted>(cancellationToken)
            .FirstOrDefault(publishedMessage => publishedMessage.Context.Message.Id == userDeletedNotification.Id.Value)
            .Should()
            .BeNull();
    }
    
    public async ValueTask DisposeAsync()
    {
        await _serviceScope.DisposeAsync();
    }
}