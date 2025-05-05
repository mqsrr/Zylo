using System.Text;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;
using Microsoft.Extensions.Options;
using Newtonsoft.Json;
using RabbitMQ.Client;
using UserManagement.Application.Settings;
using UserManagement.Application.Transport;

namespace UserManagement.Infrastructure.Transport.Producers;

internal class RabbitMqProducer<TEntity> : IProducer<TEntity> where TEntity : class
{
    private readonly IChannel _channel;
    private readonly ILogger<RabbitMqProducer<TEntity>> _logger;
    private readonly PublisherSettings _settings;

    public RabbitMqProducer(
        IBus bus,
        IServiceProvider provider,
        ILogger<RabbitMqProducer<TEntity>> logger)
    {
        _logger = logger;
        _settings = provider.GetRequiredService<IOptions<RabbitMqBusSettings>>()
            .Value
            .Publishers
            .First(q => q.AttachedType == typeof(TEntity));

        _channel = bus.GetChannel<TEntity>();
    }

    public async Task PublishAsync(TEntity message, CancellationToken cancellationToken = default)
    {
        try
        {
            await _channel.BasicPublishAsync(
                _settings.ExchangeName,
                _settings.RoutingKey,
                false,
                new BasicProperties { DeliveryMode = DeliveryModes.Persistent },
                Encoding.UTF8.GetBytes(JsonConvert.SerializeObject(message)),
                cancellationToken
            );

            _logger.LogInformation("Message of type {Type} published to exchange {Exchange} with routing key {RoutingKey}",
                typeof(TEntity).Name, _settings.ExchangeName, _settings.RoutingKey);
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Failed to publish message of type {Type}", typeof(TEntity).Name);
            throw;
        }
    }
}