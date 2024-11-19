using NotificationService.Services.Abstractions;

namespace NotificationService.HostedServices;

internal sealed class RabbitMqConsumerHostedService : BackgroundService
{
    private readonly IEnumerable<IConsumer> _consumers;
    private readonly ILogger<RabbitMqConsumerHostedService> _logger;

    public RabbitMqConsumerHostedService(IEnumerable<IConsumer> consumers, ILogger<RabbitMqConsumerHostedService> logger)
    {
        _consumers = consumers;
        _logger = logger;
    }

    protected override async Task ExecuteAsync(CancellationToken stoppingToken)
    {
        _logger.LogInformation("Starting RabbitMQ consumers");
        foreach (var consumer in _consumers)
        {
            await consumer.ConsumeAsync(stoppingToken);
        }
    }

    public override async Task StopAsync(CancellationToken cancellationToken)
    {
        _logger.LogInformation("Stopping RabbitMQ consumers");
        foreach (var consumer in _consumers)
        {
            await consumer.DisposeAsync();
        }
        
        _logger.LogInformation("All consumers are stopped");
    }
}