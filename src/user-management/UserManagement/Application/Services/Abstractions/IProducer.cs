namespace UserManagement.Application.Services.Abstractions;

public interface IProducer<in TEntity>
    where TEntity : class
{
    Task PublishAsync(TEntity message, CancellationToken cancellationToken);
}