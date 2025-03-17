using System.Data;
using Npgsql;
using UserManagement.Application.Extensions;
using UserManagement.Application.Helpers;
using UserManagement.Application.Models;
using UserManagement.Application.Models.Errors;
using UserManagement.Application.Repositories;
using UserManagement.Application.Repositories.Abstractions;

namespace UserManagement.Decorators;

internal sealed class ExceptionlessUserRepository : IUserRepository
{
    private readonly ILogger<IUserRepository> _logger;
    private readonly IUserRepository _userRepository;

    public ExceptionlessUserRepository(IUserRepository userRepository, ILogger<UserRepository> logger)
    {
        _userRepository = userRepository;
        _logger = logger;
    }

    public async Task<Result<User>> GetByIdAsync(UserId id, CancellationToken cancellationToken)
    {
        try
        {
            return await _userRepository.GetByIdAsync(id, cancellationToken); 
        }
        catch (Exception e)
        {
            _logger.LogError(e, "Unexpected error occured while getting user data by user id: {}", id);
            return new UnexpectedError(e);
        }
    }
    
    public async Task<Result<IEnumerable<UserSummary>>> GetBatchUsersSummaryByIds(IEnumerable<UserId> ids, CancellationToken cancellationToken)
    {
        try
        {
            return await _userRepository.GetBatchUsersSummaryByIds(ids, cancellationToken); 
        }
        catch (Exception e)
        {
            _logger.LogError(e, "Unexpected error occured while getting batch of users summary data: {}", ids);
            return new UnexpectedError(e);
        }
    }

    public async Task<Result> CreateAsync(User user, IDbConnection connection, IDbTransaction transaction)
    {
        try
        {
            return await _userRepository.CreateAsync(user, connection, transaction); 
        }
        catch (PostgresException e) when (e.IsUniqueViolation("username"))
        {
            return new BadRequestError("Username already exists");
        }
        catch (PostgresException e) when (e.IsForeignKeyViolation("id"))
        {
            return new BadRequestError("Identity does not exist");
        }
        catch (Exception e)
        {
            _logger.LogError(e, "Unexpected error occured during user creation: {}", user.Id);
            return new UnexpectedError(e);
        }
    }

    public async Task<Result> UpdateAsync(User user, CancellationToken cancellationToken)
    {
        try
        {
            return await _userRepository.UpdateAsync(user, cancellationToken); 
        }
        catch (Exception e)
        {
            _logger.LogError(e, "Unexpected error occured during user update: {}", user.Id);
            return new UnexpectedError(e);
        }
    }
}