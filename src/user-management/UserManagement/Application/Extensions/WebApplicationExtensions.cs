using System.Reflection;
using DbUp;
using Microsoft.Extensions.Options;
using UserManagement.Application.Settings;
using UserManagement.Application.TypeHandlers;

namespace UserManagement.Application.Extensions;

internal static class WebApplicationExtensions
{
    public static WebApplication MigrateDatabase(this WebApplication app)
    {
        string connectionString = app.Services.GetRequiredService<IOptions<PostgresDbSettings>>().Value.ConnectionString;
        EnsureDatabase.For.PostgresqlDatabase(connectionString);

        var upgradeEngine = DeployChanges.To.PostgresqlDatabase(connectionString)
            .WithScriptsEmbeddedInAssembly(Assembly.GetCallingAssembly())
            .LogToConsole()
            .Build();

        if (upgradeEngine.IsUpgradeRequired())
        {
            upgradeEngine.PerformUpgrade();
        }
        
        Dapper.SqlMapper.AddTypeHandler(new IdentityIdTypeHandler());
        Dapper.SqlMapper.AddTypeHandler(new UserIdTypeHandler());
        Dapper.SqlMapper.AddTypeHandler(new UlidTypeHandler());
        Dapper.SqlMapper.AddTypeHandler(new DateOnlyTypeHandler());
        
        return app;
    }
}