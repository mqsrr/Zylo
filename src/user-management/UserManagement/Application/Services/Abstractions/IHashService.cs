namespace UserManagement.Application.Services.Abstractions;

public interface IHashService
{
    (string Hash, string Salt) Hash(string data);

    bool VerifyHash(string data, string storedHash, string storedSalt);
}