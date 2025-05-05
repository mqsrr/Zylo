using Npgsql;

namespace UserManagement.Infrastructure.Persistence.Factories;

public interface IDbConnectionFactory
{
    Task<NpgsqlConnection> CreateAsync(CancellationToken cancellationToken);
}