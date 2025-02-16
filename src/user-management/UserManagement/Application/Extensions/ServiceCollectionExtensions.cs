using System.Collections.Concurrent;
using System.Reflection;
using Asp.Versioning;
using RabbitMQ.Client;
using StackExchange.Redis;
using UserManagement.Application.Builders;
using UserManagement.Application.Factories;
using UserManagement.Application.Factories.Abstractions;
using UserManagement.Application.Services.Abstractions;
using UserManagement.Application.Settings;
using UserManagement.HostedServices;

namespace UserManagement.Application.Extensions;

internal static class ServiceCollectionExtensions
{
    public static IServiceCollection AddOptionsSettingsWithValidation<TOptions>(
        this IServiceCollection services,
        IConfiguration configuration)
        where TOptions : BaseSettings
    {
        if (Activator.CreateInstance<TOptions>() is not BaseSettings instance)
        {
            throw new InvalidOperationException($"Could not create instance of {typeof(TOptions).Name}");
        }

        return services
            .AddOptionsWithValidateOnStart<TOptions>()
            .Bind(configuration.GetRequiredSection(instance.SectionName))
            .Services;
    }

    public static IServiceCollection AddApplicationSettings(
        this IServiceCollection services,
        IConfiguration configuration)
    {
        var baseSettingsType = typeof(BaseSettings);
        var allSettings = Assembly.GetAssembly(baseSettingsType)!
            .ExportedTypes
            .Where(t => t is { IsInterface: false, IsAbstract: false } && t.BaseType == baseSettingsType);
        
        foreach (var settings in allSettings)
        {
            var method = typeof(ServiceCollectionExtensions)
                .GetMethod(nameof(AddOptionsSettingsWithValidation))!
                .MakeGenericMethod(settings);

            method.Invoke(null, [services, configuration]);
        }

        return services;
    }

    public static IHealthChecksBuilder AddServiceHealthChecks(this IServiceCollection services, WebApplicationBuilder builder)
    {
        return services.AddHealthChecks()
            .AddRedis(builder.Configuration["Redis:ConnectionString"]!, name: "Redis", tags: ["Redis"])
            .AddNpgSql(builder.Configuration["Postgres:ConnectionString"]!, name: "Postgres", tags: ["Database"]);
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

    public static IServiceCollection AddRabbitMqBus(
        this IServiceCollection services,
        Action<RabbitMqBuilder> configure)
    {
        services.AddSingleton<ConcurrentDictionary<Type, IChannel>>();
        services.AddSingleton<IRabbitMqConnectionFactory, RabbitMqConnectionFactory>();
        services.AddSingleton<IRabbitMqBus, RabbitMqBus>();

        services.AddOptions<RabbitMqBusSettings>()
            .Configure<IServiceProvider>((settings, _) =>
            {
                var builder = new RabbitMqBuilder();
                configure(builder);
                settings.Publishers = builder.Build();
            });

        services.AddScoped(typeof(IProducer<>), typeof(RabbitMqProducer<>));
        services.AddHostedService<RabbitMqBusHostedService>();

        return services;
    }
}