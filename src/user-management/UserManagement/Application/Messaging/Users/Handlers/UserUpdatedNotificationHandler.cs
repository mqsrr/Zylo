using MassTransit;
using Mediator;

namespace UserManagement.Application.Messaging.Users.Handlers;

internal sealed class UserUpdatedNotificationHandler : INotificationHandler<UserUpdatedNotification>
{
    private readonly IPublishEndpoint _endpoint;

    public UserUpdatedNotificationHandler(IPublishEndpoint endpoint)
    {
        _endpoint = endpoint; 
    }

    public async ValueTask Handle(UserUpdatedNotification notification, CancellationToken cancellationToken)
    {
        await _endpoint.Publish<UserUpdated>(new
        {
            Id = notification.Id.Value,
            notification.Bio,
            notification.Location,
            notification.Name
        },context => context.SetRoutingKey("user.updated"), cancellationToken).ConfigureAwait(false);
    }
}