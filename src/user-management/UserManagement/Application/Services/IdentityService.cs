using UserManagement.Application.Contracts.Requests.Auth;
using UserManagement.Application.Factories.Abstractions;
using UserManagement.Application.Helpers;
using UserManagement.Application.Mappers;
using UserManagement.Application.Models;
using UserManagement.Application.Models.Errors;
using UserManagement.Application.Repositories.Abstractions;
using UserManagement.Application.Services.Abstractions;

namespace UserManagement.Application.Services;

public sealed class IdentityService : IIdentityService
{
    private readonly IDbConnectionFactory _connectionFactory;
    private readonly IHashService _hashService;
    private readonly IIdentityRepository _identityRepository;
    private readonly IOtpService _otpService;
    private readonly IUserService _userService;

    public IdentityService(IIdentityRepository identityRepository,
        IDbConnectionFactory connectionFactory, IHashService hashService, IOtpService otpService, IUserService userService)
    {
        _identityRepository = identityRepository;
        _connectionFactory = connectionFactory;
        _hashService = hashService;
        _otpService = otpService;
        _userService = userService;
    }

    public Task<Result<Identity>> GetByIdAsync(IdentityId id, CancellationToken cancellationToken)
    {
        return _identityRepository.GetByIdAsync(id, cancellationToken);
    }

    public async Task<Result<Identity>> RegisterAsync(RegisterRequest request, CancellationToken cancellationToken)
    {
        var identity = request.ToIdentity(_hashService);
        await using var connection = await _connectionFactory.CreateAsync(cancellationToken);
        await using var transaction = await connection.BeginTransactionAsync(cancellationToken);

        var result = await _identityRepository.CreateAsync(identity, connection, transaction);
        if (result.IsSuccess is false)
        {
            return result.Error;
        }

        var userCreationResult = await _userService.CreateAsync(request.ToUser(), request.ProfileImage, request.BackgroundImage, connection, transaction, cancellationToken);
        if (userCreationResult.IsSuccess is false)
        {
            return userCreationResult.Error;    
        }
        
        await transaction.CommitAsync(cancellationToken);
        return identity;
    }

    public async Task<Result<Identity>> LoginAsync(string username, string password, CancellationToken cancellationToken)
    {
        var identityResult = await _identityRepository.GetByUsernameAsync(username, cancellationToken);
        if (identityResult.IsSuccess is false)
        {
            return identityResult;
        }

        var identity = identityResult.Value!;
        bool isPasswordMatch = _hashService.VerifyHash(password, identity.PasswordHash, identity.PasswordSalt);
        return isPasswordMatch
            ? identity
            : new BadRequestError("Invalid username or password");
    }


    public async Task<Result> VerifyEmailAsync(IdentityId id, string otpCode, CancellationToken cancellationToken)
    {
        var otpResult = await _otpService.GetByIdentityIdAsync(id, cancellationToken);
        if (otpResult.IsSuccess is false)
        {
            return otpResult.Error;
        }

        var code = otpResult.Value!;
        bool codeMatch = _hashService.VerifyHash(otpCode, code.CodeHash, code.Salt);
        if (codeMatch is false || code.ExpiresAt < DateTime.UtcNow)
        {
            return new BadRequestError("Code does not match or expired");
        }
        
        var result = await _identityRepository.EmailVerifiedAsync(id, cancellationToken);
        return result;
    }

    public async Task<Result> DeleteByIdAsync(IdentityId id, CancellationToken cancellationToken)
    {
        var userDeleteResult = await _identityRepository.DeleteByIdAsync(id, cancellationToken);
        if (userDeleteResult.IsSuccess is false)
        {
            return userDeleteResult;
        }

        var imageDeleteResult = await _userService.DeleteImagesAsync(UserId.Parse(id), cancellationToken);
        return imageDeleteResult.IsSuccess
            ? Result.Success()
            : Result.Failure();
    }
}