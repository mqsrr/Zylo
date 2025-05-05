namespace UserManagement.Application.Settings;

public sealed class RabbitMqBusSettings
{
    public required List<PublisherSettings> Publishers { get; set; }
}