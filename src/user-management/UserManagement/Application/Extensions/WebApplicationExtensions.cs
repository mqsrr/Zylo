using System.Reflection;
using Dapper;
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
        
        SqlMapper.AddTypeHandler(new IdentityIdTypeHandler());
        SqlMapper.AddTypeHandler(new UserIdTypeHandler());
        SqlMapper.AddTypeHandler(new UlidTypeHandler());
        SqlMapper.AddTypeHandler(new DateOnlyTypeHandler());
        SqlMapper.AddTypeHandler(new DateTimeTypeHandler());
        
        return app;
    }
}