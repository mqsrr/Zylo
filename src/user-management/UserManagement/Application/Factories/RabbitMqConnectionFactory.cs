using Microsoft.Extensions.Options;
using RabbitMQ.Client;
using UserManagement.Application.Factories.Abstractions;
using UserManagement.Application.Settings;

namespace UserManagement.Application.Factories;

internal sealed class RabbitMqConnectionFactory : IRabbitMqConnectionFactory
{
    private readonly ConnectionFactory _connectionFactory;
    private readonly SemaphoreSlim _semaphoreSlim;
    private IConnection? _connection;

    public RabbitMqConnectionFactory(IOptions<RabbitMqSettings> settings)
    {
        _semaphoreSlim = new SemaphoreSlim(1, 1);
        _connectionFactory = new ConnectionFactory
        {
            Uri = new Uri(settings.Value.ConnectionString)
        };
    }

    public async ValueTask<IConnection> GetConnectionAsync(CancellationToken cancellationToken)
    {
        if (_connection is not null)
        {
            return _connection;
        }

        await _semaphoreSlim.WaitAsync(cancellationToken);
        try
        {
            _connection = await _connectionFactory.CreateConnectionAsync(cancellationToken);
        }
        finally
        {
            _semaphoreSlim.Release();
        }

        return _connection;
    }


    public async ValueTask DisposeAsync()
    {
        if (_connection is not null)
        {
            await _connection.CloseAsync();
        }

        _semaphoreSlim.Dispose();
    }
}