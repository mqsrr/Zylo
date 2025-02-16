using Newtonsoft.Json;
using StackExchange.Redis;
using UserManagement.Application.Services.Abstractions;

namespace UserManagement.Application.Services;

internal sealed class CacheService : ICacheService
{
    private readonly IConnectionMultiplexer _connection;

    public CacheService(IConnectionMultiplexer connection)
    {
        _connection = connection;
    }

    public async Task<TEntity?> HGetAsync<TEntity>(string key, string field) where TEntity : class
    {
        var db = _connection.GetDatabase();
        string? cachedEntity = await db.HashGetAsync(key, field);
        return cachedEntity is null
            ? null
            : JsonConvert.DeserializeObject<TEntity>(cachedEntity);
    }

    public Task<TEntity?> HFindAsync<TEntity>(string key, string pattern) where TEntity : class
    {
        var db = _connection.GetDatabase();
        var value = db.HashScan(key, pattern).FirstOrDefault();
        
        return value.Value.IsNullOrEmpty is false
            ? Task.FromResult(JsonConvert.DeserializeObject<TEntity>(value.Value!))
            : Task.FromResult<TEntity?>(null);
    }

    public async Task HSetAsync(string key, string field, string entity, TimeSpan expiry)
    {
        var db = _connection.GetDatabase();
        await db.HashSetAsync(key, field, entity);
        await db.HashFieldExpireAsync(key, [field], expiry);
    }

    public async Task HRemoveAsync(string key, string field)
    {
        var db = _connection.GetDatabase();
        await db.HashDeleteAsync(key, field);
    }

    public async Task HRemoveAllAsync(string key, string pattern)
    {
        var db = _connection.GetDatabase();
        var fields = db.HashScanNoValuesAsync(key, pattern);

        await foreach (var field in fields)
        {
            await db.HashDeleteAsync(key, field);
        }
    }

    public async Task<TEntity?> GetOrCreateAsync<TEntity>(
        string key,
        string field,
        Func<Task<TEntity?>> createEntity,
        TimeSpan? expiry) where TEntity : class
    {
        var cachedEntity = await HGetAsync<TEntity>(key, field);
        if (cachedEntity is not null)
        {
            return cachedEntity;
        }

        var entity = await createEntity();
        if (entity is null)
        {
            return null;
        }

        string jsonEntity = JsonConvert.SerializeObject(entity);

        await HSetAsync(key, field, jsonEntity, expiry ?? TimeSpan.FromMinutes(10));
        return entity;
    }
}