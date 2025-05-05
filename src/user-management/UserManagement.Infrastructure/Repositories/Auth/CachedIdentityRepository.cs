using System.Data;
using Newtonsoft.Json;
using UserManagement.Application.Common;
using UserManagement.Application.Repositories.Auth;
using UserManagement.Application.Services.Common;
using UserManagement.Domain.Auth;

namespace UserManagement.Infrastructure.Repositories.Auth;

internal sealed class CachedIdentityRepository : IIdentityRepository
{
    private readonly ICacheService _cacheService;
    private readonly IIdentityRepository _identityRepository;

    public CachedIdentityRepository(IIdentityRepository identityRepository, ICacheService cacheService)
    {
        _cacheService = cacheService;
        _identityRepository = identityRepository;
    }

    public Task<Result<Identity>> GetByIdAsync(IdentityId id, CancellationToken cancellationToken)
    {
        return _cacheService.GetOrCreateAsync("identities", id.ToString(),
            () => _identityRepository.GetByIdAsync(id, cancellationToken)!,
            TimeSpan.FromHours(1))!;
    }

    public async Task<Result<Identity>> GetByUsernameAsync(string username, CancellationToken cancellationToken)
    {
        const string cacheKey = "identities";
        var cachedIdentity = await _cacheService.HFindAsync<Identity>(cacheKey, $"*{username}");
        if (cachedIdentity is not null)
        {
            return cachedIdentity;
        }
        
        var identityResult = await _identityRepository.GetByUsernameAsync(username, cancellationToken);
        if (identityResult.IsSuccess is false)
        {
            return identityResult;
        }
        
        var identity = identityResult.Value!;
        string cacheField = $"{identity.Id}-{username}";
        
        await _cacheService.HSetAsync(cacheKey, cacheField, JsonConvert.SerializeObject(identity), TimeSpan.FromHours(1));
        return identity;
    }

    public async Task<Result> EmailVerifiedAsync(IdentityId id, CancellationToken cancellationToken)
    {
        var result = await _identityRepository.EmailVerifiedAsync(id, cancellationToken);
        if (result.IsSuccess is false)
        {
            return result;
        }

        await _cacheService.HRemoveAllAsync("identities", $"{id.Value}*");
        return result;
    }

    public Task<Result> CreateAsync(Identity identity, IDbConnection connection, IDbTransaction transaction)
    {
        return _identityRepository.CreateAsync(identity, connection, transaction);
    }

    public async Task<Result> DeleteByIdAsync(IdentityId id, CancellationToken cancellationToken)
    {
        var result = await _identityRepository.DeleteByIdAsync(id, cancellationToken);
        if (result.IsSuccess is false)
        {
            return result;
        }

        string identityId = id.ToString();
        await _cacheService.HRemoveAllAsync("identities", $"{identityId}*");
        await _cacheService.HRemoveAsync("users", identityId);
        return result;
    }
}