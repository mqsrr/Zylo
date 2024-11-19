using System.Security.Cryptography;
using Microsoft.Extensions.Options;
using UserManagement.Application.Services.Abstractions;
using UserManagement.Application.Settings;

namespace UserManagement.Application.Services;

internal sealed class OtpService : IOtpService
{
    private readonly string _characters;

    public OtpService(IOptions<OtpSettings> settings)
    {
        _characters = settings.Value.Characters;
    }

    public string CreateOneTimePassword(int length)
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
}