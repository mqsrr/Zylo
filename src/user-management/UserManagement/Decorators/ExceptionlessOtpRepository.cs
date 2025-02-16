using Npgsql;
using UserManagement.Application.Extensions;
using UserManagement.Application.Helpers;
using UserManagement.Application.Models;
using UserManagement.Application.Models.Errors;
using UserManagement.Application.Repositories.Abstractions;

namespace UserManagement.Decorators;

internal sealed class ExceptionlessOtpRepository : IOtpRepository
{
    private readonly ILogger<IOtpRepository> _logger;
    private readonly IOtpRepository _otpRepository;

    public ExceptionlessOtpRepository(IOtpRepository otpRepository, ILogger<IOtpRepository> logger)
    {
        _otpRepository = otpRepository;
        _logger = logger;
    }

    public async Task<Result> CreateAsync(OtpCode code, CancellationToken cancellationToken)
    {
        try
        {
            return await _otpRepository.CreateAsync(code, cancellationToken); 
        }
        catch (PostgresException e) when (e.IsForeignKeyViolation("id"))
        {
            return new BadRequestError("Identity does not exist");
        }
        catch (Exception e)
        {
            _logger.LogError(e, "Unexpected error occured while creating otp code");
            return new UnexpectedError(e);
        }
    }

    public async Task<Result<OtpCode>> GetByIdAsync(IdentityId id, CancellationToken cancellationToken)
    {
        try
        {
            return await _otpRepository.GetByIdAsync(id, cancellationToken); 
        }
        catch (Exception e)
        {
            _logger.LogError(e, "Unexpected error occured while getting otp code for identity: {}", id);
            return new UnexpectedError(e);
        }
    }

    public async Task<Result> DeleteByIdAsync(IdentityId id, CancellationToken cancellationToken)
    {
        try
        {
            return await _otpRepository.DeleteByIdAsync(id, cancellationToken); 
        }
        catch (Exception e)
        {
            _logger.LogError(e, "Unexpected error occured while deleting otp code for identity: {}", id);
            return new UnexpectedError(e);
        }
    }
}