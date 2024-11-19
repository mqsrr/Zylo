namespace NotificationService.Messages.User;

public sealed class UserCreated
{
    public required string Email { get; init; }
    
    public required string EmailIv { get; init; }
    
    public required string Otp { get; init; }

    public required string OtpIv { get; init; }
}