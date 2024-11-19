using AutoFixture;
using MassTransit;
using NSubstitute;
using UserManagement.Application.Messaging.Users;
using UserManagement.Application.Messaging.Users.Handlers;
using UserManagement.Application.Models;
using UserManagement.Application.Services.Abstractions;

namespace UserManagement.Tests.Unit.Messaging.Handlers;

public sealed class UserDeletedNotificationHandlerTests
{
    private readonly UserDeletedNotificationHandler _sut;
    private readonly IAuthService _authService;
    private readonly IPublishEndpoint _publishEndpoint;
    private readonly Fixture _fixture;

    public UserDeletedNotificationHandlerTests()
    {
        _fixture = new Fixture();
        _authService = Substitute.For<IAuthService>();
        _publishEndpoint = Substitute.For<IPublishEndpoint>();
        _sut = new UserDeletedNotificationHandler(_authService, _publishEndpoint);
    }

    [Fact]
    public async Task Handle_ShouldPublishUserDeleted_WhenUserIsSuccessfullyDeleted()
    {
        // Arrange
        var notification = _fixture.Build<UserDeletedNotification>()
            .With(n => n.Id, UserId.NewId())
            .Create();
        var cancellationToken = CancellationToken.None;

        _authService.DeleteByIdAsync(Arg.Is<IdentityId>(id => id.Value == notification.Id.Value), cancellationToken)
            .Returns(true);

        _publishEndpoint.Publish(Arg.Is<object>(user =>
                user.GetType().GetProperty("Id")!.GetValue(user)!.Equals(notification.Id.Value)),
            Arg.Any<IPipe<PublishContext<UserDeleted>>>(),
            cancellationToken).Returns(Task.CompletedTask);

        // Act
        await _sut.Handle(notification, cancellationToken);

        // Assert
        await _authService.Received().DeleteByIdAsync(
            Arg.Is<IdentityId>(id => id.Value == notification.Id.Value),
            cancellationToken);

        await _publishEndpoint.Received().Publish(Arg.Is<object>(user =>
                user.GetType().GetProperty("Id")!.GetValue(user)!.Equals(notification.Id.Value)),
            Arg.Any<IPipe<PublishContext<UserDeleted>>>(),
            cancellationToken);
    }

    [Fact]
    public async Task Handle_ShouldNotPublishUserDeleted_WhenUserDeletionFails()
    {
        // Arrange
        var notification = _fixture.Build<UserDeletedNotification>()
            .With(n => n.Id, UserId.NewId())
            .Create();
        
        var cancellationToken = CancellationToken.None;

        _authService.DeleteByIdAsync(Arg.Is<IdentityId>(id => id.Value == notification.Id.Value), cancellationToken)
            .Returns(false);

        // Act
        await _sut.Handle(notification, cancellationToken);

        // Assert
        await _authService.Received().DeleteByIdAsync(
            Arg.Is<IdentityId>(id => id.Value == notification.Id.Value),
            cancellationToken);

        await _publishEndpoint.DidNotReceive().Publish(Arg.Is<object>(user =>
                user.GetType().GetProperty("Id")!.GetValue(user)!.Equals(notification.Id.Value)),
            Arg.Any<IPipe<PublishContext<UserDeleted>>>(),
            cancellationToken);
    }
}