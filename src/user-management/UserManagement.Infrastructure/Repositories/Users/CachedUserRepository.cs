using System.Data;
using UserManagement.Application.Common;
using UserManagement.Application.Repositories.Users;
using UserManagement.Application.Services.Common;
using UserManagement.Domain.Users;

namespace UserManagement.Infrastructure.Repositories.Users;

internal sealed class CachedUserRepository : IUserRepository
{
    private readonly ICacheService _cacheService;
    private readonly IUserRepository _userRepository;

    public CachedUserRepository(IUserRepository userRepository, ICacheService cacheService)
    {
        _userRepository = userRepository;
        _cacheService = cacheService;
    }

    public Task<Result<User>> GetByIdAsync(UserId id, CancellationToken cancellationToken)
    {
        return _cacheService.GetOrCreateAsync("users", id.ToString(),
            () => _userRepository.GetByIdAsync(id, cancellationToken)!,
            TimeSpan.FromHours(1))!;
    }

    public Task<Result<IEnumerable<UserSummary>>> GetBatchUsersSummaryByIds(IEnumerable<UserId> ids, CancellationToken cancellationToken)
    {
        return _userRepository.GetBatchUsersSummaryByIds(ids, cancellationToken);
    }

    public Task<Result> CreateAsync(User user, IDbConnection connection, IDbTransaction transaction)
    {
        return _userRepository.CreateAsync(user, connection, transaction);
    }

    public async Task<Result> UpdateAsync(User user, CancellationToken cancellationToken)
    {
        var result = await _userRepository.UpdateAsync(user, cancellationToken);
        if (result.IsSuccess is false)
        {
            return result;
        }

        await _cacheService.HRemoveAsync("users", user.Id.ToString());
        await _cacheService.HRemoveAsync("users-summary", user.Id.ToString());
        return Result.Success();
    }
}