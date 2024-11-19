using Asp.Versioning;
using GrpcServices;
using StackExchange.Redis;
using UserManagement.Application.Attributes;

namespace UserManagement.Application.Extensions;

internal static class ServiceCollectionExtensions
{
     public static IServiceCollection AddApplicationService<TInterface>(
        this IServiceCollection services,
        ServiceLifetime serviceLifetime = ServiceLifetime.Scoped)
    {
        services.Scan(scan => scan
            .FromAssemblyOf<TInterface>()
            .AddClasses(classes =>
            {
                classes.AssignableTo<TInterface>()
                    .WithoutAttribute<Decorator>();
            })
            .AsImplementedInterfaces()
            .WithLifetime(serviceLifetime));

        return services;
    }
     
    public static IHealthChecksBuilder AddServiceHealthChecks(this IServiceCollection services, WebApplicationBuilder builder)
    {
        return services.AddHealthChecks()
            .AddRedis(builder.Configuration["Redis:ConnectionString"]!, name: "Redis", tags:["Redis"])
            .AddNpgSql(builder.Configuration["Postgres:ConnectionString"]!, name:"Postgres", tags: ["Database"]);
    }
    
    public static IApiVersioningBuilder AddApiVersioning(this IServiceCollection services, IApiVersionReader reader, bool assumeDefaultVersion = true)
    {
        return services.AddApiVersioning(x =>
            {
                x.ApiVersionReader = reader;
                x.DefaultApiVersion = new ApiVersion(1.0);
                x.ReportApiVersions = true;
                x.AssumeDefaultVersionWhenUnspecified = assumeDefaultVersion;
            })
            .AddMvc();
    }

    public static IServiceCollection AddConnectionMultiplexer(this IServiceCollection services, string connectionString)
    {
        var connection = ConnectionMultiplexer.Connect(connectionString);
        var response = connection.GetDatabase().Execute("PING");
        if (response.IsNull)
        {
            throw new Exception("Redis connection failed");
        }

        services.AddSingleton<IConnectionMultiplexer>(connection);
        return services;
    }
}