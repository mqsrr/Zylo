using Microsoft.Extensions.Options;
using UserManagement.Application.Attributes;
using UserManagement.Application.Contracts.Requests.Users;
using UserManagement.Application.Models;
using UserManagement.Application.Repositories.Abstractions;
using UserManagement.Application.Services.Abstractions;
using UserManagement.Application.Settings;

namespace UserManagement.Application.Repositories;

[Decorator]
internal sealed class CachedUserRepository : IUserRepository
{
    private readonly IUserRepository _userRepository;
    private readonly ICacheService _cacheService;
    private readonly S3Settings _s3Settings;

    public CachedUserRepository(IUserRepository userRepository, ICacheService cacheService, IOptions<S3Settings> s3Settings)
    {
        _userRepository = userRepository;
        _cacheService = cacheService;
        _s3Settings = s3Settings.Value;
    }

    public Task<User?> GetByIdAsync(UserId id, CancellationToken cancellationToken)
    {
        return _cacheService.GetOrCreateAsync("user-management",id.ToString(), async () =>
        {
            var user = await _userRepository.GetByIdAsync(id, cancellationToken);
            return user;
        }, TimeSpan.FromMinutes(_s3Settings.PresignedUrlExpire));
    }

    public Task<bool> CreateAsync(User user, IFormFile profileImage, IFormFile backgroundImage, CancellationToken cancellationToken)
    {
        return _userRepository.CreateAsync(user, profileImage, backgroundImage, cancellationToken);
    }

    public async Task<bool> UpdateAsync(UpdateUserRequest updatedUser, IFormFile profileImage, IFormFile backgroundImage, CancellationToken cancellationToken)
    {
        bool isUpdated = await _userRepository.UpdateAsync(updatedUser, profileImage, backgroundImage, cancellationToken);
        if (!isUpdated)
        {
            return false;
        }

        await _cacheService.HRemoveAsync("user-management", updatedUser.Id.ToString()).ConfigureAwait(false);
        return true;
    }

    public async Task<bool> DeleteByIdAsync(UserId id, CancellationToken cancellationToken)
    {
        bool isDeleted = await _userRepository.DeleteByIdAsync(id, cancellationToken);
        if (!isDeleted)
        {
            return false;
        }

        await _cacheService.HRemoveAsync("user-management",id.ToString()).ConfigureAwait(false);
        return true;
    }
}