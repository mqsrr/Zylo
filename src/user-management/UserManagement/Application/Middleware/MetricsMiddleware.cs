using System.Diagnostics;
using System.Diagnostics.Metrics;
using UserManagement.Application.Helpers;

namespace UserManagement.Application.Middleware;

public sealed class MetricsMiddleware : IMiddleware
{
    private readonly Instrumentation _instrumentation;
    private readonly Counter<long> _requestCount;
    private readonly Histogram<double> _requestDuration;

    public MetricsMiddleware(Instrumentation instrumentation)
    {
        _instrumentation = instrumentation;

        _requestCount = instrumentation.GetCounterOrCreate("http_server_requests_total", "Total number of HTTP requests");
        _requestDuration = instrumentation.GetHistogramOrCreate("http_server_request_duration_seconds", "HTTP request duration");

        instrumentation.RegisterGauge("http_server_active_requests", "Active HTTP requests");

    }

    public async Task InvokeAsync(HttpContext context, RequestDelegate next)
    {
        _instrumentation.IncrementGauge("http_server_active_requests", 1);
        var stopwatch = Stopwatch.StartNew();
        try
        {
            await next(context);
        }
        finally
        {
            stopwatch.Stop();

            _instrumentation.IncrementGauge("http_server_active_requests", -1);
            int statusCode = context.Response.StatusCode;
            var tagList = new TagList
            {
                { "service", Instrumentation.ActivitySourceName},
                { "host", context.Request.Host },
                { "method", context.Request.Method },
                { "path", context.Request.Path },
                { "status", statusCode },
            };
            _requestCount.Add(1,tagList);
            _requestDuration.Record(stopwatch.Elapsed.TotalSeconds,tagList);
        }
    }
}