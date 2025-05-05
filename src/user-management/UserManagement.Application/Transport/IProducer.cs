namespace UserManagement.Application.Transport;

public interface IProducer<in TEntity>
    where TEntity : class
{
    Task PublishAsync(TEntity message, CancellationToken cancellationToken);
}