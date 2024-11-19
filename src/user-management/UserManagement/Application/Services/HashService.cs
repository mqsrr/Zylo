using System.Security.Cryptography;
using UserManagement.Application.Services.Abstractions;

namespace UserManagement.Application.Services;

internal sealed class HashService : IHashService
{
    private const int SaltSize = 16;
    private const int HashSize = 32;
    private const int Iterations = 10000;
    
    public (string Hash, string Salt) Hash(string data)
    {
        byte[] salt = new byte[SaltSize];
        using var rng = RandomNumberGenerator.Create();
        
        rng.GetBytes(salt);

        using var pbkdf2 = new Rfc2898DeriveBytes(data, salt, Iterations, HashAlgorithmName.SHA256);
        byte[] hash = pbkdf2.GetBytes(HashSize);

        return (
            Convert.ToBase64String(hash), 
            Convert.ToBase64String(salt)
        );

    }

    public bool VerifyHash(string data, string storedHash, string storedSalt)
    {
        byte[] salt = Convert.FromBase64String(storedSalt);

        using var pbkdf2 = new Rfc2898DeriveBytes(data, salt, Iterations, HashAlgorithmName.SHA256);
        byte[] hash = pbkdf2.GetBytes(HashSize);

        return Convert.ToBase64String(hash) == storedHash;
    }
}