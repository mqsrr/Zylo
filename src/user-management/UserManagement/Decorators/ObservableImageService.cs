using System.Diagnostics;
using System.Diagnostics.Metrics;
using UserManagement.Application.Helpers;
using UserManagement.Application.Models;
using UserManagement.Application.Services.Abstractions;

namespace UserManagement.Decorators;

internal sealed class ObservableImageService : IImageService
{
    private readonly ActivitySource _activitySource;
    private readonly IImageService _imageService;
    private readonly Counter<long> _requestCount;
    private readonly Histogram<double> _requestDuration;
    private readonly Dictionary<string, object?> _tags;

    public ObservableImageService(IImageService imageService, Instrumentation instrumentation)
    {
        _imageService = imageService;
        _activitySource = instrumentation.ActivitySource;
        
        _requestCount = instrumentation.GetCounterOrCreate("image_service_request_count", "Number of Image service requests");
        _requestDuration = instrumentation.GetHistogramOrCreate("image_service_request_duration", "Duration of Image service requests");
        
        _tags = new Dictionary<string, object?>
        {
            ["otel.scope.name"] = "AWSSDK.S3",
            ["rpc.service"] = "S3",
            ["rpc.system"] = "aws-api",
        };
    }


    public Task<FileMetadata> GetImageAsync(UserId id, ImageCategory category, CancellationToken cancellationToken)
    {
        return ExecuteWithTelemetry(id, 
            "GetImage", 
            "GetImageAsync", 
            () => _imageService.GetImageAsync(id, category, cancellationToken),
            tags => tags["image.type"] = category.ToString());
    }

    public Task<bool> UploadImageAsync(UserId id, IFormFile file, ImageCategory category, CancellationToken cancellationToken)
    {
        return ExecuteWithTelemetry(id, 
            "UploadImage", 
            "UploadImageAsync", 
            () => _imageService.UploadImageAsync(id,file, category, cancellationToken),
            tags => tags["image.type"] = category.ToString());
    }

    public Task<bool> DeleteAllImagesAsync(UserId id, CancellationToken cancellationToken)
    {
        return ExecuteWithTelemetry(id, 
            "Delete Profile&Background Image", 
            "DeleteAllImagesAsync", 
            () => _imageService.DeleteAllImagesAsync(id, cancellationToken));
    }

    private void ReplaceTags(UserId id, string methodName)
    {
        _tags["method.name"] = methodName;
        _tags["user_id"] = id;
    }

    private async Task<T> ExecuteWithTelemetry<T>(
        UserId id,
        string operation,
        string methodName,
        Func<Task<T>> action,
        Action<Dictionary<string, object?>>? additionalTags = null)
    {
        ReplaceTags(id, methodName);
        additionalTags?.Invoke(_tags);

        using var activity = _activitySource.StartActivity(
            $"S3.{operation}",
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
            activity.SetTag("error.type", "s3_error");

            throw;
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