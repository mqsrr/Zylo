using System.Data;
using Npgsql;
using UserManagement.Application.Extensions;
using UserManagement.Application.Helpers;
using UserManagement.Application.Models;
using UserManagement.Application.Models.Errors;
using UserManagement.Application.Repositories.Abstractions;

namespace UserManagement.Decorators;

internal sealed class ExceptionlessIdentityRepository : IIdentityRepository
{
    private readonly IIdentityRepository _identityRepository;
    private readonly ILogger<IIdentityRepository> _logger;

    public ExceptionlessIdentityRepository(IIdentityRepository identityRepository, ILogger<IIdentityRepository> logger)
    {
        _identityRepository = identityRepository;
        _logger = logger;
    }

    public async Task<Result<Identity>> GetByIdAsync(IdentityId id, CancellationToken cancellationToken)
    {
        try
        {
            return await _identityRepository.GetByIdAsync(id, cancellationToken); 
        }
        catch (Exception e)
        {
            _logger.LogError(e, "Unexpected error occured while getting identity data by id: {}", id);
            return new UnexpectedError(e);
        }
    }

    public async Task<Result<Identity>> GetByUsernameAsync(string username, CancellationToken cancellationToken)
    {
        try
        {
            return await _identityRepository.GetByUsernameAsync(username, cancellationToken); 
        }
        catch (Exception e)
        {
            _logger.LogError(e, "Unexpected error occured while getting identity data by username: {}", username);
            return new UnexpectedError(e);
        }
    }

    public async Task<Result> EmailVerifiedAsync(IdentityId id, CancellationToken cancellationToken)
    {
        try
        {
            return await _identityRepository.EmailVerifiedAsync(id, cancellationToken); 
        }
        catch (Exception e)
        {
            _logger.LogError(e, "Unexpected error occured during email verification for identity: {}", id);
            return new UnexpectedError(e);
        }
    }

    public async Task<Result> CreateAsync(Identity identity, IDbConnection connection, IDbTransaction transaction)
    {
        try
        {
            return await _identityRepository.CreateAsync(identity, connection, transaction); 
        }
        catch (PostgresException e) when (e.IsUniqueViolation("username"))
        {
            return new BadRequestError("Username already exists");
        }
        catch (PostgresException e) when (e.IsUniqueViolation("email"))
        {
            return new BadRequestError("Email already exists");
        }
        catch (Exception e)
        {
            _logger.LogError(e, "Unexpected error occured during identity creation: {}", identity.Id);
            return new UnexpectedError(e);
        }
    }

    public async Task<Result> DeleteByIdAsync(IdentityId id, CancellationToken cancellationToken)
    {
        try
        {
            return await _identityRepository.DeleteByIdAsync(id, cancellationToken); 
        }
        catch (Exception e)
        {
            _logger.LogError(e, "Unexpected error occured during identity deletion for identity: {}", id);
            return new UnexpectedError(e);
        }
    }
}