namespace NotificationService.Application.Settings;

public sealed class ConsumerSettings
{
    public required Type MessageType { get; init; }

    public required string ExchangeName { get; init; }

    public required string QueueName { get; init; }

    public required string RoutingKey { get; init; }

    public required Type ConsumerType { get; init; }
}