namespace UserManagement.Application.Services.Common;

public interface IEncryptionService
{
    (string EncryptedData, string IV) Encrypt(string data);
}