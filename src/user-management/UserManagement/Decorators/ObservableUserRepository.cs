using System.Data;
using System.Diagnostics;
using System.Diagnostics.Metrics;
using UserManagement.Application.Helpers;
using UserManagement.Application.Models;
using UserManagement.Application.Repositories.Abstractions;

namespace UserManagement.Decorators;

internal sealed class ObservableUserRepository : IUserRepository
{
    private readonly ActivitySource _activitySource;
    private readonly Counter<long> _requestCount;
    private readonly Histogram<double> _requestDuration;
    private readonly Instrumentation _instrumentation;
    private readonly Dictionary<string, object?> _tags;
    private readonly IUserRepository _userRepository;

    public ObservableUserRepository(IUserRepository userRepository, Instrumentation instrumentation)
    {
        _userRepository = userRepository;
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

    public Task<Result<User>> GetByIdAsync(UserId id, CancellationToken cancellationToken)
    {
        return ExecuteWithTelemetry(
            "users",
            "SELECT",
            "GetByIdAsync", 
            () => _userRepository.GetByIdAsync(id, cancellationToken), 
            tags => tags["user_id"] = id);
    }

    public Task<Result<IEnumerable<UserSummary>>> GetBatchUsersSummaryByIds(IEnumerable<UserId> ids, CancellationToken cancellationToken)
    {
        return ExecuteWithTelemetry(
            "users",
            "SELECT id, name BATCH",
            "GetBatchByIds", 
            () => _userRepository.GetBatchUsersSummaryByIds(ids, cancellationToken));
    }

    public Task<Result> CreateAsync(User user, IDbConnection connection, IDbTransaction transaction)
    {
        return ExecuteWithTelemetry(
            "users",
            "INSERT INTO",
            "CreateAsync", 
            () => _userRepository.CreateAsync(user, connection, transaction),
            tags => tags["user_id"] = user.Id);
    }

    public Task<Result> UpdateAsync(User user, CancellationToken cancellationToken)
    {
        return ExecuteWithTelemetry(
            "users",
            "UPDATE",
            "UpdateAsync", 
            () => _userRepository.UpdateAsync(user, cancellationToken),
            tags => tags["user_id"] = user.Id);
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
                { "table", target },
                { "db", "postgres" },
                { "status", status },
            };
            
            _requestCount.Add(1, tagList);
            _requestDuration.Record(sw.Elapsed.Seconds, tagList);
        }
    }
}