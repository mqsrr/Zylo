namespace NotificationService.Services.Abstractions;

public interface IConsumer<in TEntity> : IAsyncDisposable where TEntity : class 
{
    Task ConsumeAsync(TEntity message, CancellationToken cancellationToken);
}