﻿namespace UserManagement.Application.Services.Common;

public interface IHashService
{
    (string Hash, string Salt) Hash(string data);

    bool VerifyHash(string data, string storedHash, string storedSalt);

    string CreateUniqueHash(string data);
}