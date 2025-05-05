using Microsoft.Extensions.Hosting;
using Microsoft.Extensions.Logging;
using UserManagement.Application.Transport;

namespace UserManagement.Infrastructure.Transport.HostedServices;

internal sealed class RabbitMqBusHostedService : IHostedService
{
    private readonly IBus _bus;
    private readonly ILogger<RabbitMqBusHostedService> _logger;

    public RabbitMqBusHostedService(ILogger<RabbitMqBusHostedService> logger, IBus bus)
    {
        _logger = logger;
        _bus = bus;
    }

    public async Task StartAsync(CancellationToken cancellationToken)
    {
        _logger.LogInformation("Opening RabbitMQ connection...");
        await _bus.StartAsync(cancellationToken);
        _logger.LogInformation("RabbitMq connection is successfully created.");
    }

    public async Task StopAsync(CancellationToken cancellationToken)
    {
        _logger.LogInformation("Closing RabbitMQ connection...");
        await _bus.StopAsync(cancellationToken);
        _logger.LogInformation("RabbitMq connection is successfully disposed.");
    }
}