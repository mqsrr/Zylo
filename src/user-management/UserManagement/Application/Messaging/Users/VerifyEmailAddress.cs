namespace UserManagement.Application.Messaging.Users;
public sealed class VerifyEmailAddress
{
    public required string Otp { get; init; }

    public required string OtpIv { get; init; }

    public required string Email { get; init; }

    public required string EmailIv { get; init; }
}