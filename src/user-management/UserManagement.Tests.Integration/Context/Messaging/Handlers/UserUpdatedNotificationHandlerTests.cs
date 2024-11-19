using AutoFixture;
using FluentAssertions;
using MassTransit;
using MassTransit.Testing;
using Microsoft.Extensions.DependencyInjection;
using UserManagement.Application.Messaging.Users;
using UserManagement.Application.Messaging.Users.Handlers;
using UserManagement.Application.Models;
using UserManagement.Tests.Integration.Fixtures;

namespace UserManagement.Tests.Integration.Context.Messaging.Handlers;

[Collection(nameof(UserManagementApiCollection))]
public sealed class UserUpdatedNotificationHandlerTests : IAsyncDisposable
{
    private readonly ITestHarness _testHarness;
    private readonly Fixture _fixture;
    private readonly AsyncServiceScope _serviceScope;

    private readonly UserUpdatedNotificationHandler _sut;

    public UserUpdatedNotificationHandlerTests(UserManagementApiFactory factory)
    {
        _fixture = new Fixture();

        _serviceScope = factory.Services.CreateAsyncScope();
        _testHarness = _serviceScope.ServiceProvider.GetTestHarness();

        var publishEndpoint = _serviceScope.ServiceProvider.GetRequiredService<IPublishEndpoint>();
        _sut = new UserUpdatedNotificationHandler(publishEndpoint);
    }
    
     [Fact]
    public async Task Handle_ShouldPublishUserUpdatedMessage()
    {
        // Arrange
        var cancellationToken = CancellationToken.None;
        var userUpdatedNotification = _fixture.Build<UserUpdatedNotification>()
            .With(n => n.Id, new UserId())
            .WithAutoProperties()
            .Create();
        
       
        // Act
        await _sut.Handle(userUpdatedNotification, cancellationToken);

        // Assert
        var publishedMessage = _testHarness.Published.Select<UserUpdated>(cancellationToken)
            .FirstOrDefault(publishedMessage => publishedMessage.Context.Message.Id == userUpdatedNotification.Id.Value);

        publishedMessage.Should().NotBeNull();
        publishedMessage!.Context.Message.Should().BeEquivalentTo(userUpdatedNotification, options => options.ExcludingMissingMembers());
        publishedMessage.Context.RoutingKey().Should().BeEquivalentTo("user.updated");
    }
    
    public async ValueTask DisposeAsync()
    {
        await _serviceScope.DisposeAsync();
    }
}