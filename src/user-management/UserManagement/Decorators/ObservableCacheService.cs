using System.Diagnostics;
using System.Diagnostics.Metrics;
using UserManagement.Application.Helpers;
using UserManagement.Application.Services.Abstractions;

namespace UserManagement.Decorators;

internal sealed class ObservableCacheService : ICacheService
{
    private readonly ActivitySource _activitySource;
    private readonly ICacheService _cacheService;

    private readonly Counter<long> _requestCount;
    private readonly Histogram<double> _requestDuration;
    private readonly Dictionary<string, object?> _tags;

    public ObservableCacheService(ICacheService cacheService, Instrumentation instrumentation)
    {
        _cacheService = cacheService;
        _activitySource = instrumentation.ActivitySource;

        _requestCount = instrumentation.GetCounterOrCreate("cache_service_request_count", "Number of Redis requests");
        _requestDuration = instrumentation.GetHistogramOrCreate("cache_service_request_duration", "Duration of Redis requests");
        
        _tags = new Dictionary<string, object?>
        {
            ["db.system.name"] = "redis"
        };
    }

    public Task<TEntity?> HGetAsync<TEntity>(string key, string field) where TEntity : class
    {
        return ExecuteWithTelemetry(
            key,
            "HGET",
            nameof(HGetAsync),
            () => _cacheService.HGetAsync<TEntity>(key, field),
            tags => tags["cache_field"] = field);
    }

    public Task<TEntity?> HFindAsync<TEntity>(string key, string pattern) where TEntity : class
    {
        return ExecuteWithTelemetry(
            key,
            "HSCAN HGET",
            nameof(HFindAsync),
            () => _cacheService.HFindAsync<TEntity>(key, pattern),
            tags => tags["cache_pattern"] = pattern);
    }

    public Task HSetAsync(string key, string field, string entity, TimeSpan expiry)
    {
        return ExecuteWithTelemetry(
            key,
            "HSET",
            nameof(HSetAsync),
            () => _cacheService.HSetAsync(key, field, entity, expiry),
            tags => tags["cache_field"] = field);
    }

    public Task HRemoveAsync(string key, string field)
    {
        return ExecuteWithTelemetry(
            key,
            "HDEL",
            nameof(HRemoveAsync),
            () => _cacheService.HRemoveAsync(key, field),
            tags => tags["cache_field"] = field);
    }

    public Task HRemoveAllAsync(string key, string pattern)
    {
        return ExecuteWithTelemetry(
            key,
            "HSCAN HDEL",
            nameof(HRemoveAllAsync),
            () => _cacheService.HRemoveAllAsync(key, pattern),
            tags => tags["cache_pattern"] = pattern);
    }

    public Task<TEntity?> GetOrCreateAsync<TEntity>(
        string key,
        string field,
        Func<Task<TEntity?>> createEntity,
        TimeSpan? expiry) where TEntity : class
    {
        return ExecuteWithTelemetry(
            key,
            "HGET | HSET",
            nameof(GetOrCreateAsync),
            () => _cacheService.GetOrCreateAsync(key, field, createEntity, expiry),
            tags => tags["cache_field"] = field);
    }

    private void ReplaceTags(string key, string operation, string methodName)
    {
        _tags["db.operation.name"] = operation;
        _tags["db.namespace"] = key;
        _tags["method.name"] = methodName;
        _tags["cache_key"] = key;
    }

    private async Task<T?> ExecuteWithTelemetry<T>(
        string key,
        string operation,
        string methodName,
        Func<Task<T?>> action,
        Action<Dictionary<string, object?>>? additionalTags = null) where T : class
    {
        ReplaceTags(key, operation, methodName);
        additionalTags?.Invoke(_tags);
        
        using var activity = _activitySource.StartActivity(
            $"redis.{operation} {key}",
            ActivityKind.Client,
            null,
            _tags);
        
        var sw = Stopwatch.StartNew();
        _requestCount.Add(1, new TagList
        {
            { "operation", operation },
            { "method", methodName }
        });

        try
        {
            var result = await action();
            sw.Stop();
            
            activity!.SetStatus(ActivityStatusCode.Ok);
            return result;
        }
        catch (Exception e)
        {
            activity!.SetStatus(ActivityStatusCode.Error, e.Message);
            activity.SetTag("error.message", e.Message);
            activity.SetTag("error.type", "database_error");
            
            return null;
        }
        finally
        {
            _requestDuration.Record(sw.ElapsedMilliseconds, new TagList
            {
                { "operation", operation },
                { "method", methodName },
            });
        }
    }

    private async Task ExecuteWithTelemetry(
        string key,
        string operation,
        string methodName,
        Func<Task> action,
        Action<Dictionary<string, object?>>? additionalTags = null)
    {
        await ExecuteWithTelemetry<object>(
            key,
            operation,
            methodName,
            async () => {
                await action();
                return null;
            },
            additionalTags);
    }
}