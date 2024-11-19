using AutoFixture;
using MassTransit;
using NSubstitute;
using UserManagement.Application.Messaging.Users;
using UserManagement.Application.Messaging.Users.Handlers;
using UserManagement.Application.Models;
using UserManagement.Application.Repositories.Abstractions;
using UserManagement.Tests.Unit.Fixtures;

namespace UserManagement.Tests.Unit.Messaging.Handlers;

public sealed class CreateUserNotificationHandlerTests
{
    private readonly CreateUserNotificationHandler _sut;
    private readonly IUserRepository _userRepository;
    private readonly IPublishEndpoint _publishEndpoint;
    private readonly Fixture _fixture;

    public CreateUserNotificationHandlerTests()
    {
        _fixture = new Fixture();
        _fixture.Customize(new IFormFileCustomization())
            .Customize(new DateOnlyCustomization());

        _userRepository = Substitute.For<IUserRepository>();
        _publishEndpoint = Substitute.For<IPublishEndpoint>();
        _sut = new CreateUserNotificationHandler(_userRepository, _publishEndpoint);
    }

    [Fact]
    public async Task Handle_ShouldPublishUserCreated_WhenUserIsSuccessfullyCreated()
    {
        // Arrange
        var notification = _fixture.Create<CreateUserNotification>();
        var cancellationToken = CancellationToken.None;
        
        _userRepository.CreateAsync(
                Arg.Is<User>(u => u.Id.Value == notification.Request.Id.Value),
                notification.Request.ProfileImage,
                notification.Request.BackgroundImage,
                cancellationToken)
            .Returns(true);

        _publishEndpoint.Publish(Arg.Is<object>(user =>
                user.GetType().GetProperty("Id")!.GetValue(user)!.Equals(notification.Request.Id.Value) == true &&
                user.GetType().GetProperty("Username")!.GetValue(user)!.Equals(notification.Request.Username) == true &&
                user.GetType().GetProperty("Name")!.GetValue(user)!.Equals(notification.Request.Name) == true),
            Arg.Any<IPipe<PublishContext<UserCreated>>>(),
            cancellationToken).Returns(Task.CompletedTask);
        
        // Act
        await _sut.Handle(notification, cancellationToken);

        // Assert
        await _userRepository.Received().CreateAsync(
            Arg.Is<User>(u => u.Id.Value == notification.Request.Id.Value),
            notification.Request.ProfileImage,
            notification.Request.BackgroundImage,
            cancellationToken);

        await _publishEndpoint.Received().Publish(Arg.Is<object>(user =>
                user.GetType().GetProperty("Id")!.GetValue(user)!.Equals(notification.Request.Id.Value) == true &&
                user.GetType().GetProperty("Username")!.GetValue(user)!.Equals(notification.Request.Username) == true &&
                user.GetType().GetProperty("Name")!.GetValue(user)!.Equals(notification.Request.Name) == true),
            Arg.Any<IPipe<PublishContext<UserCreated>>>(),
            cancellationToken);
    }

    [Fact]
    public async Task Handle_ShouldNotPublishUserCreated_WhenUserCreationFails()
    {
        // Arrange
        var notification = _fixture.Create<CreateUserNotification>();
        var cancellationToken = CancellationToken.None;

        _userRepository.CreateAsync(
                Arg.Is<User>(u => u.Id.Value == notification.Request.Id.Value),
                notification.Request.ProfileImage,
                notification.Request.BackgroundImage,
                cancellationToken)
            .Returns(false);

        // Act
        await _sut.Handle(notification, CancellationToken.None);

        // Assert
        await _userRepository.Received().CreateAsync(
            Arg.Is<User>(u => u.Id.Value == notification.Request.Id.Value),
            notification.Request.ProfileImage,
            notification.Request.BackgroundImage,
            cancellationToken);

        await _publishEndpoint.DidNotReceive().Publish(Arg.Is<object>(user =>
                user.GetType().GetProperty("Id")!.GetValue(user)!.Equals(notification.Request.Id.Value) == true &&
                user.GetType().GetProperty("Username")!.GetValue(user)!.Equals(notification.Request.Username) == true &&
                user.GetType().GetProperty("Name")!.GetValue(user)!.Equals(notification.Request.Name) == true),
            Arg.Any<IPipe<PublishContext<UserCreated>>>(),
            cancellationToken);
    }
}