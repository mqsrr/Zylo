namespace NotificationService.Services.Abstractions;

public interface IConsumer<in TEntity>
{
    Task ConsumeAsync(TEntity message, CancellationToken cancellationToken);
}