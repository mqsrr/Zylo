namespace NotificationService.Settings;

public sealed class RabbitMqSettings() : BaseSettings("RabbitMq")
{
    public required string ConnectionString { get; init; }
}