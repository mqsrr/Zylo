using Newtonsoft.Json;
using UserManagement.Application.Models;
using UserManagement.Application.Services.Abstractions;

namespace UserManagement.Decorators;

internal sealed class CachedImageService : IImageService
{
    private readonly ICacheService _cacheService;
    private readonly IImageService _imageService;

    public CachedImageService(IImageService imageService, ICacheService cacheService)
    {
        _imageService = imageService;
        _cacheService = cacheService;
    }

    public async Task<FileMetadata> GetImageAsync(UserId id, ImageCategory category, CancellationToken cancellationToken)
    {
        string cacheField = string.Join('-', id.ToString(), category.ToString());
        var cachedFile = await _cacheService.HGetAsync<FileMetadata>("images", cacheField);
        if (cachedFile is not null)
        {
            return cachedFile;
        }
        
        var file = await _imageService.GetImageAsync(id, category, cancellationToken);
        var expiry = file.AccessUrl.ExpiresIn - DateTime.UtcNow;
        
        await _cacheService.HSetAsync("images", cacheField, JsonConvert.SerializeObject(file), expiry);
        return file;
    }

    public async Task<bool> UploadImageAsync(UserId id, IFormFile file, ImageCategory category, CancellationToken cancellationToken)
    {
        bool isCreated = await _imageService.UploadImageAsync(id, file, category, cancellationToken);
        if (isCreated is false)
        {
            return false;
        }
        
        string cacheField = string.Join('-', id.ToString(), category.ToString());
        await _cacheService.HRemoveAsync("images", cacheField);
        
        return true;
    }

    public async Task<bool> DeleteAllImagesAsync(UserId id, CancellationToken cancellationToken)
    {
        bool isDeleted = await _imageService.DeleteAllImagesAsync(id, cancellationToken);
        if (isDeleted is false)
        {
            return false;
        }
        
        await _cacheService.HRemoveAllAsync("images", $"{id}*");
        return true;
    }
}