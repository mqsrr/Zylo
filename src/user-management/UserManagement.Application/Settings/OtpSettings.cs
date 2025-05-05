namespace UserManagement.Application.Settings;

public sealed class OtpSettings() : BaseSettings("Otp")
{
    public required string Characters { get; init; }
}