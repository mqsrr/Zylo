namespace UserManagement.Application.Models;

public record struct UserId(Ulid Value)
{
    public static UserId NewId()
    {
        return new UserId(Ulid.NewUlid());
    }

    public static UserId Parse(string uid)
    {
        return new UserId(Ulid.Parse(uid));
    }

    public static UserId Parse(IdentityId id)
    {
        return new UserId(id.Value);
    }

    public override string ToString()
    {
        return Value.ToString();   
    }
}

public sealed class User
{
    public required UserId Id { get; init; }

    public FileMetadata? ProfileImage { get; set; }

    public FileMetadata? BackgroundImage { get; set; }

    public required string Name { get; init; }

    public string? Username { get; init; }

    public string? Bio { get; init; }

    public string? Location { get; init; }

    public required DateOnly BirthDate { get; init; }
}