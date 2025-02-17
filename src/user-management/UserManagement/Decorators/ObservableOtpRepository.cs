using System.Diagnostics;
using System.Diagnostics.Metrics;
using UserManagement.Application.Helpers;
using UserManagement.Application.Models;
using UserManagement.Application.Repositories.Abstractions;

namespace UserManagement.Decorators;

internal sealed class ObservableOtpRepository : IOtpRepository
{
    private readonly ActivitySource _activitySource;
    private readonly IOtpRepository _otpRepository;
    private readonly Counter<long> _requestCount;
    private readonly Histogram<double> _requestDuration;
    private readonly Dictionary<string, object?> _tags;

    public ObservableOtpRepository(IOtpRepository otpRepository, Instrumentation instrumentation)
    {
        _otpRepository = otpRepository;
        _activitySource = instrumentation.ActivitySource;
        
        _requestCount = instrumentation.GetCounterOrCreate("otp_repository_request_count", "Number of Otp repository requests");
        _requestDuration = instrumentation.GetHistogramOrCreate("otp_repository_request_duration", "Duration of Otp repository requests");
        
        _tags = new Dictionary<string, object?>
        {
            ["db.system.name"] = "postgresql"
        };
    }


    public Task<Result<OtpCode>> GetByIdAsync(IdentityId id, CancellationToken cancellationToken)
    {
        return ExecuteWithTelemetry(
            "otp",
            "SELECT",
            "GetByIdAsync", 
            () => _otpRepository.GetByIdAsync(id, cancellationToken), 
            tags => tags["identity_id"] = id);
    }

    public Task<Result> CreateAsync(OtpCode code, CancellationToken cancellationToken)
    {
        return ExecuteWithTelemetry(
            "otp",
            "INSERT INTO",
            "CreateAsync", 
            () => _otpRepository.CreateAsync(code, cancellationToken), 
            tags => tags["identity_id"] = code.Id);
    }

    public Task<Result> DeleteByIdAsync(IdentityId id, CancellationToken cancellationToken)
    {
        return ExecuteWithTelemetry(
            "otp",
            "DELETE FROM",
            "DeleteByIdAsync", 
            () => _otpRepository.DeleteByIdAsync(id, cancellationToken), 
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
        
        _requestCount.Add(1, new TagList
        {
            { "operation", operation },
            { "method", methodName }
        });
        
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
            
            return null!;
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
}