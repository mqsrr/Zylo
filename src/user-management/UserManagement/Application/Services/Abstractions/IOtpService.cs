using System.Security.Cryptography;
using Microsoft.Extensions.Options;
using UserManagement.Application.Helpers;
using UserManagement.Application.Messaging.Users;
using UserManagement.Application.Models;
using UserManagement.Application.Repositories.Abstractions;
using UserManagement.Application.Settings;

namespace UserManagement.Application.Services.Abstractions;

public interface IOtpService
{
    Task<Result<OtpCode>> CreateAsync(IdentityId id, int length, string email, CancellationToken cancellationToken);

    Task<Result<OtpCode>> GetByIdentityIdAsync(IdentityId id, CancellationToken cancellationToken);

    Task<Result> DeleteByIdentityIdAsync(IdentityId id, CancellationToken cancellationToken);
}

public class OtpService : IOtpService
{
    private readonly string _characters;
    private readonly IEncryptionService _encryptionService;
    private readonly IHashService _hashService;
    private readonly IOtpRepository _otpRepository;
    private readonly IProducer<VerifyEmailAddress> _producer;

    public OtpService(IOptions<OtpSettings> settings, IHashService hashService, IOtpRepository otpRepository, IProducer<VerifyEmailAddress> producer, IEncryptionService encryptionService)
    {
        _hashService = hashService;
        _otpRepository = otpRepository;
        _producer = producer;
        _encryptionService = encryptionService;
        _characters = settings.Value.Characters;
    }

    public async Task<Result<OtpCode>> CreateAsync(IdentityId id, int length, string email, CancellationToken cancellationToken)
    {
        string code = CreateOneTimePassword(length);
        (string hashedCode, string salt) = _hashService.Hash(code);

        var otpCode = CreateOtpCodeFromIdentityId(id, hashedCode, salt);
        var creationResult = await _otpRepository.CreateAsync(otpCode, cancellationToken);
        if (creationResult.IsSuccess is false)
        {
            return creationResult.Error;
        }
        
        (string encryptedCode, string otpIv) = _encryptionService.Encrypt(code);
        (string encryptedEmail, string emailIv) = _encryptionService.Encrypt(email);

        await _producer.PublishAsync(new VerifyEmailAddress
        {
            Otp = encryptedCode,
            OtpIv = otpIv,
            Email = encryptedEmail,
            EmailIv = emailIv
        }, cancellationToken);
        
        return creationResult.IsSuccess is false
            ? creationResult.Error
            : otpCode;
    }

    public Task<Result<OtpCode>> GetByIdentityIdAsync(IdentityId id, CancellationToken cancellationToken)
    {
        return _otpRepository.GetByIdAsync(id, cancellationToken);
    }

    public Task<Result> DeleteByIdentityIdAsync(IdentityId id, CancellationToken cancellationToken)
    {
        return _otpRepository.DeleteByIdAsync(id, cancellationToken);
    }

    private string CreateOneTimePassword(int length)
    {
        Span<char> otp = stackalloc char[length];
        Span<byte> randomBytes = stackalloc byte[length];

        using var rng = RandomNumberGenerator.Create();
        rng.GetBytes(randomBytes);

        for (int i = 0; i < length; i++)
        {
            otp[i] = _characters[randomBytes[i] % _characters.Length];
        }

        return otp.ToString();
    }

    private static OtpCode CreateOtpCodeFromIdentityId(IdentityId id, string codeHash, string salt)
    {
        return new OtpCode
        {
            Id = id,
            CodeHash = codeHash,
            Salt = salt,
            ExpiresAt = DateTime.UtcNow.AddMonths(1),
            CreatedAt = DateTime.UtcNow
        };
    }
}