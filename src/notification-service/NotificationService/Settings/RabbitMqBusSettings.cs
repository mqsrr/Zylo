namespace NotificationService.Settings;

public class RabbitMqBusSettings
{
    public required List<PublisherSettings> Publishers { get; set; }

    public required List<ConsumerSettings> Consumers { get; set; }
}