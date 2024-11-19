namespace UserManagement.Application.Settings;

internal sealed class RabbitMqSettings
{
    public const string SectionName = "RabbitMq";

    public required string ConnectionString { get; init; }
}