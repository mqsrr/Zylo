using System.Diagnostics;
using System.Diagnostics.Metrics;
using UserManagement.Application.Helpers;
using UserManagement.Application.Models;
using UserManagement.Application.Repositories.Abstractions;

namespace UserManagement.Decorators;

internal sealed class ObservableRefreshTokenRepository : IRefreshTokenRepository
{
    private readonly ActivitySource _activitySource;
    private readonly Counter<long> _requestCount;
    private readonly Histogram<double> _requestDuration;
    private readonly Instrumentation _instrumentation;
    private readonly Dictionary<string, object?> _tags;
    private readonly IRefreshTokenRepository _tokenRepository;

    public ObservableRefreshTokenRepository(IRefreshTokenRepository tokenRepository, Instrumentation instrumentation)
    {
        _tokenRepository = tokenRepository;
        _instrumentation = instrumentation;
        _activitySource = instrumentation.ActivitySource;
        
        _requestCount = instrumentation.GetCounterOrCreate("db_queries_total", "Total number of database queries");
        _requestDuration = instrumentation.GetHistogramOrCreate("db_query_duration_seconds", "Query execution duration");

        instrumentation.RegisterGauge("db_connections", "Active database connections");

        _tags = new Dictionary<string, object?>
        {
            ["db.system.name"] = "postgresql"
        };
    }


    public Task<Result<RefreshToken>> GetByIdentityIdAsync(IdentityId id, CancellationToken cancellationToken)
    {
        return ExecuteWithTelemetry(
            "refresh_tokens",
            "SELECT",
            "GetByIdentityIdAsync", 
            () => _tokenRepository.GetByIdentityIdAsync(id, cancellationToken), 
            tags => tags["identity_id"] = id);
    }

    public Task<Result<RefreshToken>> GetRefreshTokenAsync(byte[] refreshToken, CancellationToken cancellationToken)
    {
        return ExecuteWithTelemetry(
            "refresh_tokens",
            "SELECT",
            "GetRefreshTokenAsync", 
            () => _tokenRepository.GetRefreshTokenAsync(refreshToken, cancellationToken));
    }

    public Task<Result> CreateAsync(RefreshToken refreshToken, CancellationToken cancellationToken)
    {
        return ExecuteWithTelemetry(
            "refresh_tokens",
            "INSERT INTO",
            "CreateAsync", 
            () => _tokenRepository.CreateAsync(refreshToken, cancellationToken), 
            tags => tags["identity_id"] = refreshToken.IdentityId);
    }

    public Task<Result> DeleteAsync(byte[] refreshToken, CancellationToken cancellationToken)
    {
        return ExecuteWithTelemetry(
            "refresh_tokens",
            "DELETE FROM",
            "DeleteAsync", 
            () => _tokenRepository.DeleteAsync(refreshToken, cancellationToken));
    }

    public Task<Result> DeleteAllByIdAsync(IdentityId id, CancellationToken cancellationToken)
    {
        return ExecuteWithTelemetry(
            "refresh_tokens",
            "DELETE FROM",
            "DeleteAllByIdAsync", 
            () => _tokenRepository.DeleteAllByIdAsync(id, cancellationToken), 
            tags => tags["identity_id"] = id);
    }

    private void ReplaceTags(string target, string operation, string methodName)
    {
        _tags["db.operation.name"] = operation;
        _tags["db.target"] = target;
        _tags["method.name"] = methodName;
    }

    private async Task<T> ExecuteWithTelemetry<T>(
        string target,
        string operation,
        string methodName,
        Func<Task<T>> action,
        Action<Dictionary<string, object?>>? additionalTags = null) where T : class
    {
        ReplaceTags(target, operation, methodName);
        additionalTags?.Invoke(_tags);

        using var activity = _activitySource.StartActivity(
            $"postgres.{operation} {target}",
            ActivityKind.Client,
            null,
            _tags);
        
        string status = "success";
        _instrumentation.IncrementGauge("db_connections", 1);
        var sw = Stopwatch.StartNew();
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
            status = "error";
            
            throw;
        }
        finally
        {
            _instrumentation.IncrementGauge("db_connections", -1);
            var tagList = new TagList
            {
                { "service", Instrumentation.ActivitySourceName},
                { "operation", operation },
                { "method", methodName },
                { "db", "postgres" },
                { "status", status },
            };
            
            _requestCount.Add(1, tagList);
            _requestDuration.Record(sw.Elapsed.Seconds, tagList);
        }
    }
}