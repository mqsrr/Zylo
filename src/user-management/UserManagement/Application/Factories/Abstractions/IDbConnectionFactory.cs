using Npgsql;

namespace UserManagement.Application.Factories.Abstractions;

public interface IDbConnectionFactory
{
    Task<NpgsqlConnection> CreateAsync(CancellationToken cancellationToken);
}