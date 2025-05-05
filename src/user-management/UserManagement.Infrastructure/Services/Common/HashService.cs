using System.Security.Cryptography;
using System.Text;
using Microsoft.Extensions.Options;
using UserManagement.Application.Services.Common;
using UserManagement.Application.Settings;

namespace UserManagement.Infrastructure.Services.Common;

internal sealed class HashService : IHashService
{
    private const int SaltSize = 32;
    private const int HashSize = 32;
    private const int Iterations = 350000;
    private readonly string _pepper;

    public HashService(IOptions<HashSettings> settings)
    {
        _pepper = settings.Value.Pepper;
    }

    public (string Hash, string Salt) Hash(string data)
    {
        byte[] salt = RandomNumberGenerator.GetBytes(SaltSize);

        using var pbkdf2 = new Rfc2898DeriveBytes(
            data, 
            salt, 
            Iterations, 
            HashAlgorithmName.SHA512);
            
        byte[] hash = pbkdf2.GetBytes(HashSize);

        return (
            Convert.ToBase64String(hash), 
            Convert.ToBase64String(salt)
        );
    }

    public string CreateUniqueHash(string data)
    {
        string normalizedData = data.Trim().ToLowerInvariant();

        string pepperedData = string.Concat(normalizedData, _pepper);
        byte[] hashBytes = SHA512.HashData(Encoding.UTF8.GetBytes(pepperedData));

        return Convert.ToBase64String(hashBytes.Take(32).ToArray());
    }

    public bool VerifyHash(string data, string storedHash, string storedSalt)
    {
        byte[] salt = Convert.FromBase64String(storedSalt);

        using var pbkdf2 = new Rfc2898DeriveBytes(
            data, 
            salt, 
            Iterations, 
            HashAlgorithmName.SHA512);
            
        byte[] hash = pbkdf2.GetBytes(HashSize);
        return Convert.ToBase64String(hash) == storedHash;
    }
}