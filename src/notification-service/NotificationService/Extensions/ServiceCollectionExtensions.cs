using Asp.Versioning;
using OpenTelemetry.Logs;
using OpenTelemetry.Metrics;
using OpenTelemetry.Resources;
using OpenTelemetry.Trace;

namespace NotificationService.Extensions;

internal static class ServiceCollectionExtensions
{
    public static IServiceCollection ConfigureOpenTelemetry(this IServiceCollection services, string collectorAddress, ILoggingBuilder loggingBuilder)
    {
        loggingBuilder.AddOpenTelemetry(options => options.AddOtlpExporter(exporterOptions => exporterOptions.Endpoint = new Uri(collectorAddress)));

        services.AddOpenTelemetry()
            .ConfigureResource(resourceBuilder => resourceBuilder.AddTelemetrySdk().AddService(serviceName: "notification-service", serviceVersion: "1.0.0"))
            .WithMetrics(providerBuilder =>
            {
                providerBuilder.AddRuntimeInstrumentation()
                    .AddHttpClientInstrumentation()
                    .AddOtlpExporter(options => options.Endpoint = new Uri(collectorAddress));
            })
            .WithTracing(providerBuilder =>
            {
                providerBuilder
                    .AddHttpClientInstrumentation()
                    .AddOtlpExporter(options => options.Endpoint = new Uri(collectorAddress));
            });

        return services;
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
}