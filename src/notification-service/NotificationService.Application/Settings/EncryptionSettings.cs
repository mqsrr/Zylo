namespace NotificationService.Application.Settings;

public sealed class EncryptionSettings() : BaseSettings("Encryption")
{
    public required string Key { get; init; }
}