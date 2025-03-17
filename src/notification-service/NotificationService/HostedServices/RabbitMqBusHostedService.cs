using NotificationService.Services.Abstractions;

namespace NotificationService.HostedServices;

internal sealed class RabbitMqBusHostedService : IHostedService
{
    private readonly IRabbitMqBus _bus;
    private readonly ILogger<RabbitMqBusHostedService> _logger;

    public RabbitMqBusHostedService(ILogger<RabbitMqBusHostedService> logger, IRabbitMqBus bus)
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