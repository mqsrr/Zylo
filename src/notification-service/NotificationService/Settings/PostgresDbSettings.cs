namespace NotificationService.Settings;

public sealed class PostgresDbSettings(): BaseSettings("Postgres")
{
    public required string ConnectionString { get; init; }
}