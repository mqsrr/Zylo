using RabbitMQ.Client;

namespace NotificationService.Services.Abstractions;

public interface IRabbitMqBus
{
    Task StartAsync(CancellationToken cancellationToken);

    Task StopAsync(CancellationToken cancellationToken);

    IChannel GetChannel<TEntity>() where TEntity : class;
}