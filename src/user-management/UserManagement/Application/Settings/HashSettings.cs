namespace UserManagement.Application.Settings;

public sealed class HashSettings() : BaseSettings("Hash")
{
    public required string Pepper { get; init; }
}