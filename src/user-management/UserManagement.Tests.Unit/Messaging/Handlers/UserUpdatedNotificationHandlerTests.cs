using AutoFixture;
using MassTransit;
using NSubstitute;
using UserManagement.Application.Messaging.Users;
using UserManagement.Application.Messaging.Users.Handlers;
using UserManagement.Application.Models;

namespace UserManagement.Tests.Unit.Messaging.Handlers;


public sealed class UserUpdatedNotificationHandlerTests
{
    private readonly UserUpdatedNotificationHandler _sut;
    private readonly IPublishEndpoint _publishEndpoint;
    private readonly Fixture _fixture;

    public UserUpdatedNotificationHandlerTests()
    {
        _fixture = new Fixture();
        _publishEndpoint = Substitute.For<IPublishEndpoint>();
        _sut = new UserUpdatedNotificationHandler(_publishEndpoint);
    }

    [Fact]
    public async Task Handle_ShouldPublishUserUpdated_WithRoutingKey()
    {
        // Arrange
        var notification = _fixture.Build<UserUpdatedNotification>()
            .With(n => n.Id, UserId.NewId())
            .WithAutoProperties()
            .Create();
        
        var cancellationToken = CancellationToken.None;

        _publishEndpoint.Publish(Arg.Is<object>(user =>
                user.GetType().GetProperty("Id")!.GetValue(user)!.Equals(notification.Id.Value) &&
                user.GetType().GetProperty("Bio")!.GetValue(user)!.Equals(notification.Bio) &&
                user.GetType().GetProperty("Location")!.GetValue(user)!.Equals(notification.Location) &&
                user.GetType().GetProperty("Name")!.GetValue(user)!.Equals(notification.Name)),
            Arg.Any<IPipe<PublishContext<UserUpdated>>>(),
            cancellationToken).Returns(Task.CompletedTask);

        // Act
        await _sut.Handle(notification, cancellationToken);

        // Assert
        await _publishEndpoint.Received().Publish(Arg.Is<object>(user =>
                user.GetType().GetProperty("Id")!.GetValue(user)!.Equals(notification.Id.Value) &&
                user.GetType().GetProperty("Bio")!.GetValue(user)!.Equals(notification.Bio) &&
                user.GetType().GetProperty("Location")!.GetValue(user)!.Equals(notification.Location) &&
                user.GetType().GetProperty("Name")!.GetValue(user)!.Equals(notification.Name)),
            Arg.Any<IPipe<PublishContext<UserUpdated>>>(),
            cancellationToken);
    }
}