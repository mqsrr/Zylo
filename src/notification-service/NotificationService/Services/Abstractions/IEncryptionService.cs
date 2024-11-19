namespace NotificationService.Services.Abstractions;

public interface IEncryptionService
{
    (string EncryptedData, string IV) Encrypt(string data);
    
    string Decrypt(string encryptedData, string base64Iv);
}