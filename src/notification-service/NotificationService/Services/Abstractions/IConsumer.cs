namespace NotificationService.Services.Abstractions;

public interface IConsumer : IAsyncDisposable
{
    Task ConsumeAsync(CancellationToken cancellationToken);
}