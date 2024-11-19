using System.Data;
using Dapper;
using Mediator;
using Npgsql;
using UserManagement.Application.Contracts.Requests.Users;
using UserManagement.Application.Factories.Abstractions;
using UserManagement.Application.Helpers;
using UserManagement.Application.Messaging.Users;
using UserManagement.Application.Models;
using UserManagement.Application.Repositories.Abstractions;
using UserManagement.Application.Services.Abstractions;

namespace UserManagement.Application.Repositories;

internal sealed class UserRepository : IUserRepository
{
    private readonly IDbConnectionFactory _dbConnectionFactory;
    private readonly IImageService _imageService;
    private readonly IPublisher _publisher;

    public UserRepository(IDbConnectionFactory dbConnectionFactory, IPublisher publisher, IImageService imageService)
    {
        _dbConnectionFactory = dbConnectionFactory;
        _publisher = publisher;
        _imageService = imageService;
    }

    public async Task<User?> GetByIdAsync(UserId id, CancellationToken cancellationToken)
    {
        await using var connection = await _dbConnectionFactory.CreateAsync(cancellationToken);
        await using var transaction = await connection.BeginTransactionAsync(IsolationLevel.ReadCommitted, cancellationToken);
        try
        {
            var user = await connection.QueryFirstOrDefaultAsync<User>(SqlQueries.Users.GetById, new { Id = id.Value }, transaction);
            if (user is null)
            {
                return null;
            }
        
            await transaction.CommitAsync(cancellationToken);
            user.ProfileImage = await _imageService.GetImageAsync(id, ImageCategory.Profile, cancellationToken);
            user.BackgroundImage = await _imageService.GetImageAsync(id, ImageCategory.Background, cancellationToken);
            
            return user;

        }
        catch (PostgresException)
        {
            await transaction.RollbackAsync(cancellationToken);
            return null;
        }
    }

    public async Task<bool> CreateAsync(User user, IFormFile profileImage, IFormFile backgroundImage, CancellationToken cancellationToken)
    {
        await using var connection = await _dbConnectionFactory.CreateAsync(cancellationToken);
        await using var transaction = await connection.BeginTransactionAsync(IsolationLevel.Serializable, cancellationToken);
        try
        {
            int affectedRows = await connection.ExecuteAsync(SqlQueries.Users.Create, user, transaction);
            if (affectedRows < 1)
            {
                return false;
            }
            await transaction.CommitAsync(cancellationToken);

            var profileUpload = _imageService.UploadImageAsync(user.Id, profileImage, ImageCategory.Profile, cancellationToken);
            var backgroundUpload =  _imageService.UploadImageAsync(user.Id, backgroundImage, ImageCategory.Background, cancellationToken);
        
            bool[] result = await Task.WhenAll(profileUpload, backgroundUpload);
            return result.All(r => r);
        }
        catch (PostgresException)
        {
            await transaction.RollbackAsync(cancellationToken);
            return false;
        }
    }

    public async Task<bool> UpdateAsync(UpdateUserRequest updatedUser, IFormFile profileImage, IFormFile backgroundImage, CancellationToken cancellationToken)
    {
        await using var connection = await _dbConnectionFactory.CreateAsync(cancellationToken);
        await using var transaction = await connection.BeginTransactionAsync(IsolationLevel.RepeatableRead, cancellationToken);
        try
        {
            int affectedRows = await connection.ExecuteAsync(SqlQueries.Users.Update, updatedUser, transaction);
            if (affectedRows < 1)
            {
                return false;
            }
            
            await transaction.CommitAsync(cancellationToken);
            
            var profileUpload = _imageService.UploadImageAsync(updatedUser.Id, profileImage, ImageCategory.Profile, cancellationToken);
            var backgroundUpload =  _imageService.UploadImageAsync(updatedUser.Id, backgroundImage, ImageCategory.Background, cancellationToken);
        
            bool[] result = await Task.WhenAll(profileUpload, backgroundUpload);
            if (!result.All(r => r))
            {
                return false;
            }

            await _publisher.Publish(new UserUpdatedNotification
            {
                Id = updatedUser.Id,
                Name = updatedUser.Name,
                Bio = updatedUser.Bio,
                Location = updatedUser.Location
            }, cancellationToken).ConfigureAwait(false);
            return true;
        }
        catch (PostgresException)
        {
            await transaction.RollbackAsync(cancellationToken);
            return false;
        }
    }

    public async Task<bool> DeleteByIdAsync(UserId id, CancellationToken cancellationToken)
    {
        await using var connection = await _dbConnectionFactory.CreateAsync(cancellationToken);
        await using var transaction = await connection.BeginTransactionAsync(IsolationLevel.Serializable, cancellationToken);
        try
        {
            int affectedRows = await connection.ExecuteAsync(SqlQueries.Users.DeleteById, new { Id = id }, transaction);
            if (affectedRows <= 0)
            {
                return false;
            }

            await transaction.CommitAsync(cancellationToken);
            await _publisher.Publish(new UserDeletedNotification
            {
                Id = id
            }, cancellationToken).ConfigureAwait(false);

            await _imageService.DeleteAllImagesAsync(id, cancellationToken);
            return true;
        }
        catch (PostgresException)
        {
            await transaction.RollbackAsync(cancellationToken);
            return false;
        }
    }
}