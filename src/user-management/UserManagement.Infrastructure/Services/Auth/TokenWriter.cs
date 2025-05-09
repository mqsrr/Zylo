﻿using System.Buffers;
using System.IdentityModel.Tokens.Jwt;
using System.Security.Claims;
using System.Security.Cryptography;
using System.Text;
using Microsoft.Extensions.Options;
using Microsoft.IdentityModel.Tokens;
using UserManagement.Application.Services.Auth;
using UserManagement.Application.Settings;
using UserManagement.Domain.Auth;

namespace UserManagement.Infrastructure.Services.Auth;

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
        
        var currentTime = DateTime.SpecifyKind(_timeProvider.GetUtcNow().DateTime, DateTimeKind.Utc);
        var expiresAt = DateTime.SpecifyKind(currentTime.AddSeconds(_jwtSettings.Expire), DateTimeKind.Utc);
        
        var jwtSecurityToken = new JwtSecurityToken(
            _jwtSettings.Issuer,
            _jwtSettings.Audience,
            [
                new Claim(JwtRegisteredClaimNames.Sub, identity.Id.ToString()),
                new Claim("email_verified", identity.EmailVerified.ToString())
            ],
            currentTime,
            expiresAt ,
            signingCred);

        return new AccessToken
        {
            Value = new JwtSecurityTokenHandler().WriteToken(jwtSecurityToken),
            ExpirationDate = expiresAt
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
            IdentityId = id,
            ExpiresAt = _timeProvider.GetUtcNow().DateTime.AddDays(30),
        };
    }
}