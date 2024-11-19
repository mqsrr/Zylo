namespace UserManagement.Application.Settings;

public sealed class OtpSettings
{
    public const string SectionName = "Otp";
    
    public required string Characters { get; set; }
    
}