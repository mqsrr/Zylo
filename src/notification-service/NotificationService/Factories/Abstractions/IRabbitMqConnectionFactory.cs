using RabbitMQ.Client;

namespace NotificationService.Factories.Abstractions;

public interface IRabbitMqConnectionFactory : IAsyncDisposable
{
    ValueTask<IConnection> GetConnectionAsync(CancellationToken cancellationToken);
}