using System.Text;
using Microsoft.AspNetCore.SignalR;
using Newtonsoft.Json;
using NotificationService.Factories.Abstractions;
using NotificationService.Hubs;
using NotificationService.Services.Abstractions;
using RabbitMQ.Client;
using RabbitMQ.Client.Events;

namespace NotificationService.Messages.Follower.Consumers;

internal sealed class UserFollowedConsumer : IConsumer
{
    private readonly IRabbitMqConnectionFactory _connectionFactory;
    private readonly IHubContext<NotificationHub, INotificationHub> _hubContext;
    private IChannel? _channel;

    public UserFollowedConsumer(IRabbitMqConnectionFactory connectionFactory, IHubContext<NotificationHub, INotificationHub> hubContext)
    {
        _connectionFactory = connectionFactory;
        _hubContext = hubContext;
    }

    public async Task ConsumeAsync(CancellationToken cancellationToken)
    {
        var connection = await _connectionFactory.GetConnectionAsync(cancellationToken);
        _channel = await  connection.CreateChannelAsync(null, cancellationToken);

        await _channel.ExchangeDeclareAsync("user-exchange", "direct", true, false, cancellationToken: cancellationToken);
        var queueDeclare= await _channel.QueueDeclareAsync("user-followed-notification-service", true, false, false, cancellationToken: cancellationToken);
        
        await _channel.QueueBindAsync(queueDeclare.QueueName, "user-exchange", "user.followed", cancellationToken: cancellationToken);
        
        var consumer = new AsyncEventingBasicConsumer(_channel);
        consumer.ReceivedAsync += async (_, ea) =>
        {
            var userFollowed = JsonConvert.DeserializeObject<UserFollowed>(Encoding.UTF8.GetString(ea.Body.ToArray()));
            if (userFollowed is null)
            {
                await _channel.BasicNackAsync(ea.DeliveryTag, false, false, cancellationToken);
                return;
            }

            await _hubContext.Clients.Group(userFollowed.FollowedId).UserFollowed(userFollowed.Id);
            await _channel.BasicAckAsync(ea.DeliveryTag, false, ea.CancellationToken);
        };

        await _channel.BasicConsumeAsync(queueDeclare.QueueName,false, consumer, cancellationToken);
    }

    public async ValueTask DisposeAsync()
    {
        if (_channel is not null)
        {
            await _channel.DisposeAsync();
        }
        await _connectionFactory.DisposeAsync();
    }

}