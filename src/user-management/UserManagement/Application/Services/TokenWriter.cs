using System.Buffers;
using System.IdentityModel.Tokens.Jwt;
using System.Security.Claims;
using System.Security.Cryptography;
using System.Text;
using Microsoft.Extensions.Options;
using Microsoft.IdentityModel.Tokens;
using UserManagement.Application.Models;
using UserManagement.Application.Services.Abstractions;
using UserManagement.Application.Settings;

namespace UserManagement.Application.Services;

internal sealed class  TokenWriter : ITokenWriter
{
    private readonly JwtSettings _jwtSettings;
    private readonly TimeProvider _timeProvider;
    
    public TokenWriter(IOptions<JwtSettings> jwtSettings, TimeProvider timeProvider)
    {
        _jwtSettings = jwtSettings.Value;
        _timeProvider = timeProvider;
    }

    public byte[]? ParseRefreshToken(string? tokenString)
    {
        if (string.IsNullOrEmpty(tokenString))
        {
            return null;
        }
        
        Span<byte> buffer = stackalloc byte[1024];
        return Convert.TryFromBase64String(tokenString, buffer, out int bytesWritten)
            ? buffer[..bytesWritten].ToArray()
            : null;
    }

    public AccessToken GenerateAccessToken(Identity identity)
    {
        var symmetricKey = new SymmetricSecurityKey(Encoding.UTF8.GetBytes(_jwtSettings.Secret));
        var signingCred = new SigningCredentials(symmetricKey, SecurityAlgorithms.HmacSha256);
        var currentTime = _timeProvider.GetUtcNow().DateTime;
        
        var jwtSecurityToken = new JwtSecurityToken(
            _jwtSettings.Issuer,
            _jwtSettings.Audience,
            [
                new Claim(JwtRegisteredClaimNames.Sub, identity.Id.ToString()),
                new Claim("email_verified", identity.EmailVerified.ToString())
            ],
            currentTime,
            currentTime.AddMinutes(_jwtSettings.Expire),
            signingCred);

        return new AccessToken
        {
            Value = new JwtSecurityTokenHandler().WriteToken(jwtSecurityToken),
            ExpirationDate = jwtSecurityToken.ValidTo
        };
    }
    
    public RefreshToken GenerateRefreshToken(IdentityId id)
    {
        byte[] randomNumber = ArrayPool<byte>.Shared.Rent(64);
        using var rng = RandomNumberGenerator.Create();
        
        rng.GetBytes(randomNumber);
        
        ArrayPool<byte>.Shared.Return(randomNumber);
        return new RefreshToken
        {
            Token = randomNumber,
            IdentityId = id.Value,
            ExpirationDate = _timeProvider.GetUtcNow().DateTime.AddDays(30)
        };
    }
}