using MassTransit;
using Mediator;
using UserManagement.Application.Models;
using UserManagement.Application.Services.Abstractions;

namespace UserManagement.Application.Messaging.Users.Handlers;

internal sealed class UserDeletedNotificationHandler : INotificationHandler<UserDeletedNotification>
{
    private readonly IAuthService _authService;
    private readonly IPublishEndpoint _publishEndpoint;

    public UserDeletedNotificationHandler(IAuthService authService, IPublishEndpoint publishEndpoint)
    {
        _authService = authService;
        _publishEndpoint = publishEndpoint;
    }

    public async ValueTask Handle(UserDeletedNotification notification, CancellationToken cancellationToken)
    {
        bool isDeleted = await _authService.DeleteByIdAsync(new IdentityId(notification.Id.Value), cancellationToken);
        if (isDeleted)
        {
            await _publishEndpoint.Publish<UserDeleted>(new
            {
                Id = notification.Id.Value       
            }, context => context.SetRoutingKey("user.deleted"), cancellationToken).ConfigureAwait(false);
        }
    }
}