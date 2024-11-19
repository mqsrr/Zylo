﻿using UserManagement.Application.Models;

namespace UserManagement.Application.Services.Abstractions;


public enum ImageCategory
{
    Profile,
    Background
}

public interface IImageService
{
    Task<FileMetadata> GetImageAsync(UserId id, ImageCategory category, CancellationToken cancellationToken);
    
    Task<bool> UploadImageAsync(UserId id, IFormFile file, ImageCategory category, CancellationToken cancellationToken);
    
    Task<bool> DeleteAllImagesAsync(UserId id, CancellationToken cancellationToken);
}