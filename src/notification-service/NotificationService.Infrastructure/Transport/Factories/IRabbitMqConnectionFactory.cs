using RabbitMQ.Client;

namespace NotificationService.Infrastructure.Transport.Factories;

public interface IRabbitMqConnectionFactory : IAsyncDisposable
{
    ValueTask<IConnection> GetConnectionAsync(CancellationToken cancellationToken);
}