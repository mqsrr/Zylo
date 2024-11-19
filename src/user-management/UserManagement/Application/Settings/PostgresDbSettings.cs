namespace UserManagement.Application.Settings;

internal sealed class PostgresDbSettings
{
    public const string SectionName = "Postgres";
    
    public required string ConnectionString { get; init; }
    
}