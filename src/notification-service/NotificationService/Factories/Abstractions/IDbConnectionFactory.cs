using Npgsql;

namespace NotificationService.Factories.Abstractions;

public interface IDbConnectionFactory
{
    Task<NpgsqlConnection> CreateAsync(CancellationToken cancellationToken);
}