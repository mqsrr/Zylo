using RabbitMQ.Client;

namespace UserManagement.Application.Factories.Abstractions;

public interface IRabbitMqConnectionFactory : IAsyncDisposable
{
    ValueTask<IConnection> GetConnectionAsync(CancellationToken cancellationToken);
}