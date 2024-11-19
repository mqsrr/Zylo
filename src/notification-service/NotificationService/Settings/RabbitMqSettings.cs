namespace NotificationService.Settings;

public sealed class RabbitMqSettings
{
    public const string SectionName = "RabbitMq";
    
    public required string ConnectionString { get; init; }
}