namespace UserManagement.Application.Settings;

public sealed class EncryptionSettings
{
    public const string SectionName = "Encryption";
    
    public required string Key { get; init; }
}