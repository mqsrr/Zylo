using RabbitMQ.Client;

namespace UserManagement.Infrastructure.Transport.Factories;

public interface IRabbitMqConnectionFactory : IAsyncDisposable
{
    ValueTask<IConnection> GetConnectionAsync(CancellationToken cancellationToken);
}