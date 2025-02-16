﻿using System.Data;
using UserManagement.Application.Helpers;
using UserManagement.Application.Messaging.Users;
using UserManagement.Application.Models;
using UserManagement.Application.Repositories.Abstractions;
using UserManagement.Application.Services.Abstractions;

namespace UserManagement.Application.Services;

internal sealed class UserService : IUserService
{
    private readonly IImageService _imageService;
    private readonly IProducer<UserCreated> _userCreatedProducer;
    private readonly IProducer<UserDeleted> _userDeletedProducer;
    private readonly IUserRepository _userRepository;

    public UserService(IUserRepository userRepository, IImageService imageService, IProducer<UserDeleted> userDeletedProducer, IProducer<UserCreated> userCreatedProducer)
    {
        _userRepository = userRepository;
        _imageService = imageService;
        _userDeletedProducer = userDeletedProducer;
        _userCreatedProducer = userCreatedProducer;
    }

    public async Task<Result<User>> GetByIdAsync(UserId id, CancellationToken cancellationToken)
    {
        var userResult = await _userRepository.GetByIdAsync(id, cancellationToken);
        if (userResult.IsSuccess is false)
        {
            return userResult;
        }

        var user = userResult.Value!;
        user.ProfileImage = await _imageService.GetImageAsync(user.Id, ImageCategory.Profile, cancellationToken);
        user.BackgroundImage = await _imageService.GetImageAsync(user.Id, ImageCategory.Background, cancellationToken);

        return user;
    }

    public async Task<Result<IEnumerable<User>>> GetBatchUsersByIdsAsync(IEnumerable<UserId> ids, CancellationToken cancellationToken)
    {
        var usersResult = await _userRepository.GetBatchByIds(ids, cancellationToken);
        if (usersResult.IsSuccess is false)
        {
            return usersResult;
        }

        var userTasks = new Dictionary<UserId, (Task<FileMetadata> ProfileTask, Task<FileMetadata> BackgroundTask)>();
        var users = usersResult.Value!.ToList();
        
        foreach (var user in users)
        {
            var profileImageTask = _imageService.GetImageAsync(user.Id, ImageCategory.Profile, cancellationToken);
            var backgroundImageTask = _imageService.GetImageAsync(user.Id, ImageCategory.Background, cancellationToken);
            
            userTasks.Add(user.Id, (profileImageTask, backgroundImageTask));
        }

        await Task.WhenAll(userTasks.Values.SelectMany(t => new[] { t.ProfileTask, t.BackgroundTask }));
        foreach (var user in users)
        {
            user.ProfileImage = await userTasks[user.Id].ProfileTask;
            user.BackgroundImage = await userTasks[user.Id].BackgroundTask;
        }

        return users;
    }

    public async Task<Result> CreateAsync(User user, IFormFile profileImage, IFormFile backgroundImage, IDbConnection connection, IDbTransaction transaction,
        CancellationToken cancellationToken)
    {
        var creationResult = await _userRepository.CreateAsync(user, connection, transaction);
        if (creationResult.IsSuccess is false)
        {
            return creationResult;
        }

        var profileUploadResult = _imageService.UploadImageAsync(user.Id, profileImage, ImageCategory.Profile, cancellationToken);
        var backgroundUploadResult = _imageService.UploadImageAsync(user.Id, backgroundImage, ImageCategory.Background, cancellationToken);
        
        bool[] isSuccess = await Task.WhenAll(profileUploadResult, backgroundUploadResult);
        if (isSuccess.Any(v => v is false))
        {
            return Result.Failure();
        }

        await _userCreatedProducer.PublishAsync(new UserCreated
        {
            Id = user.Id
        }, cancellationToken);

        return Result.Success();
    }

    public async Task<Result<User>> UpdateAsync(User user,IFormFile? profileImage, IFormFile? backgroundImage, CancellationToken cancellationToken)
    {
        var updateResult = await _userRepository.UpdateAsync(user, cancellationToken);
        if (updateResult.IsSuccess is false)
        {
            return updateResult.Error;
        }

        if (profileImage is not null)
        {
            await _imageService.UploadImageAsync(user.Id, profileImage, ImageCategory.Profile, cancellationToken);
        }
        
        if (backgroundImage is not null)
        {
            await _imageService.UploadImageAsync(user.Id, backgroundImage, ImageCategory.Background, cancellationToken);
        }
        
        user.ProfileImage = await _imageService.GetImageAsync(user.Id, ImageCategory.Profile, cancellationToken);
        user.BackgroundImage = await _imageService.GetImageAsync(user.Id, ImageCategory.Background, cancellationToken);
        return user;
    }

    public async Task<Result> DeleteImagesAsync(UserId id, CancellationToken cancellationToken)
    {
        bool isImagesDeleted = await _imageService.DeleteAllImagesAsync(id, cancellationToken);
        if (isImagesDeleted is false)
        {
            return Result.Failure();
        }
        
        await _userDeletedProducer.PublishAsync(new UserDeleted
        {
            Id = id
        }, cancellationToken);

        return Result.Success();
    }
}