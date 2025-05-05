namespace NotificationService.Domain.Users;

public record struct UserId(Ulid Value)
{
    public static UserId Parse(string uid)
    {
        return new UserId(Ulid.Parse(uid));
    }
}

public sealed class User
{
    public required UserId Id { get; init; }

    public required string Email { get; init; }

    public required string EmailIv { get; init; }
}