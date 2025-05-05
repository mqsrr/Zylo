namespace UserManagement.Application.Services.Common;

public interface ICacheService
{
    public Task<TEntity?> HGetAsync<TEntity>(string key, string field) where TEntity : class;

    public Task<TEntity?> HFindAsync<TEntity>(string key, string pattern) where TEntity : class;

    Task HSetAsync(string key, string field, string entity, TimeSpan expiry);

    Task HRemoveAsync(string key, string field);

    Task HRemoveAllAsync(string key, string pattern);

    Task<TEntity?> GetOrCreateAsync<TEntity>(string key, string field, Func<Task<TEntity?>> createEntity, TimeSpan? expiry) where TEntity : class;
}