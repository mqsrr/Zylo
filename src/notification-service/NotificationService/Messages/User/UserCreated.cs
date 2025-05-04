using NotificationService.Models;

namespace NotificationService.Messages.User;

internal sealed class UserCreated
{
    public required UserId Id { get; init; }

    public required string Email { get; init; }

    public required string EmailIv { get; init; }

    public required string Otp { get; init; }

    public required string OtpIv { get; init; }
}