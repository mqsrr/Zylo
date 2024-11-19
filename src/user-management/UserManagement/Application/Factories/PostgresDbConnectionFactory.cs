using Microsoft.Extensions.Options;
using Npgsql;
using UserManagement.Application.Factories.Abstractions;
using UserManagement.Application.Settings;

namespace UserManagement.Application.Factories;

internal sealed class PostgresDbConnectionFactory : IDbConnectionFactory
{
    private readonly PostgresDbSettings _dbSettings;

    public PostgresDbConnectionFactory(IOptions<PostgresDbSettings> dbSettings)
    {
        _dbSettings = dbSettings.Value;
    }

    public async Task<NpgsqlConnection> CreateAsync(CancellationToken cancellationToken)
    {
        var connection = new NpgsqlConnection(_dbSettings.ConnectionString);
        await connection.OpenAsync(cancellationToken);

        return connection;
    }
}