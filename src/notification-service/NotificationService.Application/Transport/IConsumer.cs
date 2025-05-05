namespace NotificationService.Application.Transport;

public interface IConsumer<in TEntity>
{
    Task ConsumeAsync(TEntity message, CancellationToken cancellationToken);
}