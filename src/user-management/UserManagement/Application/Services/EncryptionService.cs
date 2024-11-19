using System.Security.Cryptography;
using System.Text;
using Microsoft.Extensions.Options;
using UserManagement.Application.Services.Abstractions;
using UserManagement.Application.Settings;

namespace UserManagement.Application.Services;

internal sealed class EncryptionService : IEncryptionService
{
    private readonly string _encryptionKey;

    public EncryptionService(IOptions<EncryptionSettings> settings)
    {
        _encryptionKey = settings.Value.Key;
    }
    
    public (string EncryptedData, string IV) Encrypt(string data)
    {
        using var aes = Aes.Create();
        
        byte[] keyBytes = Convert.FromBase64String(_encryptionKey);
        byte[] hashedKey = SHA256.HashData(keyBytes);
        aes.Key = hashedKey;
        
        aes.GenerateIV();

        using var encryptor = aes.CreateEncryptor();
        using var ms = new MemoryStream();
        using var cs = new CryptoStream(ms, encryptor, CryptoStreamMode.Write);

        byte[] plaintextBytes = Encoding.UTF8.GetBytes(data);
        cs.Write(plaintextBytes, 0, plaintextBytes.Length);
        cs.FlushFinalBlock();

        return (Convert.ToBase64String(ms.ToArray()), Convert.ToBase64String(aes.IV));
    }

    public string Decrypt(string encryptedData, string base64Iv)
    {
        byte[] ciphertext = Convert.FromBase64String(encryptedData);
        byte[] iv = Convert.FromBase64String(base64Iv);

        using var aes = Aes.Create();
        aes.Key = Convert.FromBase64String(_encryptionKey);;
        aes.IV = iv;

        using var decryptor = aes.CreateDecryptor();
        using var ms = new MemoryStream(ciphertext);
        using var cs = new CryptoStream(ms, decryptor, CryptoStreamMode.Read);
        using var reader = new StreamReader(cs);
        
        return reader.ReadToEnd();
    }
}