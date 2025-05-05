
using RabbitMQ.Client;

namespace UserManagement.Application.Transport;

public interface IBus
{
    Task StartAsync(CancellationToken cancellationToken);

    Task StopAsync(CancellationToken cancellationToken);

    IChannel GetChannel<TEntity>() where TEntity : class;
}