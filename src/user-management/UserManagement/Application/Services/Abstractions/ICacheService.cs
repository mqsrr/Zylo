namespace UserManagement.Application.Services.Abstractions;

public interface ICacheService
{
    public Task<TEntity?> HGetAsync<TEntity>(string key, string field) where TEntity : class;

    Task HSetAsync(string key, string field, string entity, TimeSpan expiry);
    
    Task HRemoveAsync(string key, string fields);
    
    Task<TEntity?> GetOrCreateAsync<TEntity>(string key, string field, Func<Task<TEntity?>> createEntity, TimeSpan? expiry) where TEntity : class;
}