using UserManagement.Domain.Auth;

namespace UserManagement.Application.Messages;

public sealed class VerifyEmailAddress
{
    public required IdentityId Id { get; init; }

    public required string Otp { get; init; }

    public required string OtpIv { get; init; }

    public required string Email { get; init; }

    public required string EmailIv { get; init; }
}