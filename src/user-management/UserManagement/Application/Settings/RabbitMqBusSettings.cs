namespace UserManagement.Application.Settings;

public class RabbitMqBusSettings
{
    public required List<PublisherSettings> Publishers { get; set; }
}