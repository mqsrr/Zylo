using MassTransit;
using Mediator;
using UserManagement.Application.Mappers;
using UserManagement.Application.Repositories.Abstractions;

namespace UserManagement.Application.Messaging.Users.Handlers;

internal sealed class CreateUserNotificationHandler : INotificationHandler<CreateUserNotification>
{
    private readonly IUserRepository _userRepository;
    private readonly IPublishEndpoint _publishEndpoint;

    public CreateUserNotificationHandler(IUserRepository userRepository, IPublishEndpoint publishEndpoint)
    {
        _userRepository = userRepository;
        _publishEndpoint = publishEndpoint;
    }

    public async ValueTask Handle(CreateUserNotification notification, CancellationToken cancellationToken)
    {
        bool isCreated = await _userRepository.CreateAsync(notification.Request.ToUser(), notification.Request.ProfileImage, 
            notification.Request.BackgroundImage, cancellationToken);

        if (!isCreated)
        {
            return;
        }

        await _publishEndpoint.Publish<UserCreated>(new
        {
            Id = notification.Request.Id.Value,
            notification.Request.Username,
            notification.Request.Name
        }, context => context.SetRoutingKey("user.created"), cancellationToken).ConfigureAwait(false);

    }
}