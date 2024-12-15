namespace UserManagement.Application.Services.Abstractions;

public interface IEncryptionService
{
    (string EncryptedData, string IV) Encrypt(string data);
}