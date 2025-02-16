using System.Collections.Concurrent;
using Microsoft.Extensions.Options;
using RabbitMQ.Client;
using UserManagement.Application.Factories.Abstractions;
using UserManagement.Application.Settings;

namespace UserManagement.Application.Services.Abstractions;

public interface IRabbitMqBus
{
    Task StartAsync(CancellationToken cancellationToken);

    Task StopAsync(CancellationToken cancellationToken);

    IChannel GetChannel<TEntity>() where TEntity : class;
}

internal sealed class RabbitMqBus : IRabbitMqBus, IAsyncDisposable
{
    private readonly ConcurrentDictionary<Type, IChannel> _channels;
    private readonly IRabbitMqConnectionFactory _connectionFactory;
    private readonly RabbitMqBusSettings _settings;

    public RabbitMqBus(IRabbitMqConnectionFactory connectionFactory, ConcurrentDictionary<Type, IChannel> channels, IOptions<RabbitMqBusSettings> settings)
    {
        _connectionFactory = connectionFactory;
        _channels = channels;
        _settings = settings.Value;
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
            await channel.ExchangeDeclarePassiveAsync(publisherSettings.ExchangeName, cancellationToken);
            _channels.TryAdd(publisherSettings.AttachedType, channel);
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
            throw new NullReferenceException($"No publisher registered for message type {messageType.Name}");    
        }
        return channel;
    }
}