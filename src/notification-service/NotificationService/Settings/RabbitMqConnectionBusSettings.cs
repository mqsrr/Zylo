namespace NotificationService.Settings;

public sealed class PublisherSettings
{
    public required string ExchangeName { get; init; }

    public required string RoutingKey { get; init; }

    public required Type AttachedType { get; init; }
}