﻿using System.Data;
using System.Diagnostics;
using System.Diagnostics.Metrics;
using UserManagement.Application.Common;
using UserManagement.Application.Repositories.Auth;
using UserManagement.Domain.Auth;
using UserManagement.Infrastructure.Services.Common;

namespace UserManagement.Infrastructure.Repositories.Auth;

public sealed class ObservableIdentityRepository : IIdentityRepository
{
    private readonly ActivitySource _activitySource;
    private readonly IIdentityRepository _identityRepository;
    private readonly Instrumentation _instrumentation;
    private readonly Counter<long> _requestCount;
    private readonly Histogram<double> _requestDuration;
    private readonly Dictionary<string, object?> _tags;


    public ObservableIdentityRepository(IIdentityRepository identityRepository, Instrumentation instrumentation)
    {
        _identityRepository = identityRepository;
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

    public Task<Result<Identity>> GetByIdAsync(IdentityId id, CancellationToken cancellationToken)
    {
        return ExecuteWithTelemetry(
            "identities",
            "SELECT",
            "GetByIdAsync", 
            () => _identityRepository.GetByIdAsync(id, cancellationToken), 
            tags => tags["identity_id"] = id);
    }

    public Task<Result<Identity>> GetByUsernameAsync(string username, CancellationToken cancellationToken)
    {
        return ExecuteWithTelemetry(
            "identities",
            "SELECT",
            "GetByIdAsync", 
            () => _identityRepository.GetByUsernameAsync(username, cancellationToken), 
            tags => tags["identity_username"] = username);
    }

    public Task<Result> EmailVerifiedAsync(IdentityId id, CancellationToken cancellationToken)
    {
        return ExecuteWithTelemetry(
            "identities.email_verified",
            "UPDATE",
            "EmailVerifiedAsync", 
            () => _identityRepository.EmailVerifiedAsync(id, cancellationToken), 
            tags => tags["identity_id"] = id);
    }

    public Task<Result> CreateAsync(Identity identity, IDbConnection connection, IDbTransaction transaction)
    {
        return ExecuteWithTelemetry(
            "identities",
            "INSERT INTO",
            "CreateAsync", 
            () => _identityRepository.CreateAsync(identity, connection, transaction), 
            tags => tags["identity_id"] = identity.Id);
    }

    public Task<Result> DeleteByIdAsync(IdentityId id, CancellationToken cancellationToken)
    {
        return ExecuteWithTelemetry(
            "identities",
            "DELETE FROM",
            "DeleteByIdAsync", 
            () => _identityRepository.DeleteByIdAsync(id, cancellationToken), 
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
            Activity.Current?.Id,
            _tags);


        string status = "success";
        _instrumentation.IncrementGauge("db_connections", 1);

        var sw = Stopwatch.StartNew();
        try
        {
            var result = await action();
            sw.Stop();
            
            activity?.SetStatus(ActivityStatusCode.Ok);
            return result;
        }
        catch (Exception e)
        {
            activity?.SetStatus(ActivityStatusCode.Error, e.Message);
            activity?.SetTag("error.message", e.Message);
            activity?.SetTag("error.type", "database_error");
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