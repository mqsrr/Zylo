using System.Collections.Concurrent;
using System.Reflection;
using Asp.Versioning;
using Newtonsoft.Json;
using Newtonsoft.Json.Serialization;
using OpenTelemetry.Logs;
using OpenTelemetry.Metrics;
using OpenTelemetry.Resources;
using OpenTelemetry.Trace;
using RabbitMQ.Client;
using StackExchange.Redis;
using UserManagement.Application.Builders;
using UserManagement.Application.Factories;
using UserManagement.Application.Factories.Abstractions;
using UserManagement.Application.Helpers;
using UserManagement.Application.Models;
using UserManagement.Application.Services;
using UserManagement.Application.Services.Abstractions;
using UserManagement.Application.Settings;
using UserManagement.HostedServices;

namespace UserManagement.Application.Extensions;

internal static class ServiceCollectionExtensions
{
    public static WebApplicationBuilder ConfigureOpenTelemetry(this WebApplicationBuilder builder, string collectorAddress)
    {

        builder.Logging.AddOpenTelemetry(options => options.AddOtlpExporter(exporterOptions => exporterOptions.Endpoint = new Uri(collectorAddress)));
        var boundaries = new ExplicitBucketHistogramConfiguration
        {
            Boundaries = [0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
        };
        builder.Services.AddOpenTelemetry()
            .ConfigureResource(resourceBuilder => resourceBuilder.AddTelemetrySdk().AddService(serviceName: "user-management", serviceVersion: "1.0.0"))
            .WithMetrics(providerBuilder =>
            {
                providerBuilder.AddRuntimeInstrumentation()
                    .AddHttpClientInstrumentation()
                    .AddAWSInstrumentation()
                    .AddProcessInstrumentation()
                    .AddAspNetCoreInstrumentation()
                    .AddMeter(Instrumentation.MeterName, "Microsoft.AspNetCore.Hosting", "Microsoft.AspNetCore.Server.Kestrel", "System.Net.Http")
                    .AddView("http_server_request_duration_seconds", boundaries)
                    .AddView("db_query_duration_seconds", boundaries)
                    .AddView("grpc_server_request_duration_seconds", boundaries)
                    .AddView("s3_request_duration_seconds", boundaries)
                    .AddOtlpExporter(options => options.Endpoint = new Uri(collectorAddress));
            })
            .WithTracing(providerBuilder =>
            {
                providerBuilder.AddAspNetCoreInstrumentation(options =>
                    {
                        options.EnrichWithHttpResponse = (activity, httpRequest) => activity.SetTag("http.request.id", httpRequest.HttpContext.TraceIdentifier);
                    })
                    .AddHttpClientInstrumentation()
                    .AddAWSInstrumentation()
                    .AddOtlpExporter(options => options.Endpoint = new Uri(collectorAddress));
            });

        builder.Services.AddSingleton<Instrumentation>();
        return builder;
    }

    public static IServiceCollection ConfigureJsonSerializer(this IServiceCollection services)
    {
        JsonConvert.DefaultSettings = () => new JsonSerializerSettings
        {
            Formatting = Formatting.Indented,
            TypeNameHandling = TypeNameHandling.None,
            ContractResolver = new CamelCasePropertyNamesContractResolver(),
            Converters = [new UserIdConverter()]
        };

        return services;
    }

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