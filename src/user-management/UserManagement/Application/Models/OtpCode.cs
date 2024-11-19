﻿namespace UserManagement.Application.Models;

public sealed class OtpCode
{
    public required IdentityId Id { get; init; }

    public required string CodeHash { get; init; }

    public required string Salt { get; init; }

    public required DateTime CreatedAt { get; init; }

    public required DateTime ExpiresAt { get; init; }
}