using System.Collections.Concurrent;
using System.Text;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;
using Microsoft.Extensions.Options;
using Newtonsoft.Json;
using NotificationService.Application.Settings;
using NotificationService.Application.Transport;
using NotificationService.Infrastructure.Transport.Factories;
using RabbitMQ.Client;
using RabbitMQ.Client.Events;

namespace NotificationService.Infrastructure.Transport.Bus;

internal sealed class RabbitMqBus : IBus, IAsyncDisposable
{
    private readonly ConcurrentDictionary<Type, IChannel> _channels;
    private readonly IRabbitMqConnectionFactory _connectionFactory;
    private readonly RabbitMqBusSettings _settings;
    private readonly IServiceScopeFactory _serviceProvider;
    private readonly ILogger<RabbitMqBus> _logger;

    public RabbitMqBus(IRabbitMqConnectionFactory connectionFactory, ConcurrentDictionary<Type, IChannel> channels, IOptions<RabbitMqBusSettings> settings, IServiceScopeFactory serviceProvider, ILogger<RabbitMqBus> logger)
    {
        _connectionFactory = connectionFactory;
        _channels = channels;
        _settings = settings.Value;
        _serviceProvider = serviceProvider;
        _logger = logger;
    }

    public async ValueTask DisposeAsync()
    {
        await StopAsync(CancellationToken.None);
    }

    public async Task StartAsync(CancellationToken cancellationToken)
    {
        var connection = await _connectionFactory.GetConnectionAsync(cancellationToken);
        foreach (var publisherSettings in _settings.Publishers)
        {
            var channel = await connection.CreateChannelAsync(new CreateChannelOptions(true, true), cancellationToken);
            await channel.ExchangeDeclareAsync(publisherSettings.ExchangeName,"direct", true, false, null, false, false, cancellationToken);
            _channels.TryAdd(publisherSettings.AttachedType, channel);
        }

        foreach (var consumerSettings in _settings.Consumers)
        {
            var channel = await connection.CreateChannelAsync(new CreateChannelOptions(true, true), cancellationToken);
            await channel.ExchangeDeclareAsync(consumerSettings.ExchangeName, "direct", true, false, cancellationToken: cancellationToken);
            var queueDeclare = await channel.QueueDeclareAsync(consumerSettings.QueueName, true, false, false, cancellationToken: cancellationToken);
            await channel.QueueBindAsync(queueDeclare.QueueName, consumerSettings.ExchangeName, consumerSettings.RoutingKey, cancellationToken: cancellationToken);

            var eventConsumer = new AsyncEventingBasicConsumer(channel);
            eventConsumer.ReceivedAsync += async (_, ea) =>
            {
                using var scope = _serviceProvider.CreateScope();
                try
                {
                    string messageJson = Encoding.UTF8.GetString(ea.Body.ToArray());
                    var messageType = consumerSettings.MessageType;
                    object? message = JsonConvert.DeserializeObject(messageJson, messageType);
                    if (message is null)
                    {
                        await channel.BasicNackAsync(ea.DeliveryTag, false, false, cancellationToken);
                        return;
                    }

                    var consumerType = typeof(IConsumer<>).MakeGenericType(messageType);
                    object consumer = scope.ServiceProvider.GetRequiredService(consumerType);

                    var consumeMethod = consumerType.GetMethod("ConsumeAsync");
                    if (consumeMethod is null)
                    {
                        throw new Exception("ConsumeAsync method not found on consumer.");
                    }

                    var task = (Task)consumeMethod.Invoke(consumer, [message, cancellationToken])!;
                    await task;

                    await channel.BasicAckAsync(ea.DeliveryTag, false, cancellationToken);
                }
                catch (Exception err)
                {
                    _logger.LogError(err,"Error consuming message");
                    await channel.BasicNackAsync(ea.DeliveryTag, false, false, cancellationToken);
                }
            };

            await channel.BasicConsumeAsync(queueDeclare.QueueName, false, eventConsumer, cancellationToken);
        }
    }

    public async Task StopAsync(CancellationToken cancellationToken)
    {
        foreach (var channel in _channels.Values)
        {
            await channel.CloseAsync(cancellationToken);
        }
        
        _channels.Clear();
        await _connectionFactory.DisposeAsync();
    }

    public IChannel GetChannel<TEntity>() where TEntity : class
    {
        var messageType = typeof(TEntity);
        if (!_channels.TryGetValue(messageType, out var channel))
        {
            throw new NullReferenceException($"No register for message type {messageType.Name}");    
        }
        return channel;
    }
}