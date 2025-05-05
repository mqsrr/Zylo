namespace NotificationService.Application.Settings;

public sealed class RabbitMqBusSettings
{
    public required List<PublisherSettings> Publishers { get; set; }

    public required List<ConsumerSettings> Consumers { get; set; }
}