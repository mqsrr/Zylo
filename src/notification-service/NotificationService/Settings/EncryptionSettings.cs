namespace NotificationService.Settings;

internal sealed class EncryptionSettings() : BaseSettings("Encryption")
{
    public required string Key { get; init; }
}