using Asp.Versioning;
using OpenTelemetry.Logs;
using OpenTelemetry.Metrics;
using OpenTelemetry.Resources;
using OpenTelemetry.Trace;
using UserManagement.Infrastructure.Services.Common;

namespace UserManagement.Application.Extensions;

internal static class ServiceCollectionExtensions
{
    public static IServiceCollection ConfigureOpenTelemetry(this IServiceCollection services, string collectorAddress, ILoggingBuilder loggingBuilder)
    {

        loggingBuilder.AddOpenTelemetry(options => options.AddOtlpExporter(exporterOptions => exporterOptions.Endpoint = new Uri(collectorAddress)));
        var boundaries = new ExplicitBucketHistogramConfiguration
        {
            Boundaries = [0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
        };

        services.AddOpenTelemetry()
            .ConfigureResource(resourceBuilder => resourceBuilder.AddTelemetrySdk().AddService(serviceName: "user-management", serviceVersion: "1.0.0"))
            .WithMetrics(providerBuilder =>
            {
                providerBuilder.AddRuntimeInstrumentation()
                    .AddHttpClientInstrumentation()
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
                    .AddOtlpExporter(options => options.Endpoint = new Uri(collectorAddress));
            });

        services.AddSingleton<Instrumentation>();
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