using NotificationService.Domain.Users;

namespace NotificationService.Application.Messages;

internal sealed class UserCreated
{
    public required UserId Id { get; init; }

    public required string Email { get; init; }

    public required string EmailIv { get; init; }

    public required string Otp { get; init; }

    public required string OtpIv { get; init; }
}