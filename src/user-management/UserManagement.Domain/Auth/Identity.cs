namespace UserManagement.Domain.Auth;

public readonly record struct IdentityId(Ulid Value)
{
    public static IdentityId NewId()
    {
        return new IdentityId(Ulid.NewUlid());
    }

    public static IdentityId Parse(string id)
    {
        return new IdentityId(Ulid.Parse(id));
    }

    public override string ToString()
    {
        return Value.ToString();   
    }
}

public sealed class Identity
{
    public required IdentityId Id { get; init; }

    public required string Username { get; init; }

    public required string EmailHash { get; init; }

    public required string EmailSalt { get; init; }

    public required string EmailUniqueHash { get; init; }

    public required bool EmailVerified { get; init; }

    public required string PasswordHash { get; init; }

    public required string PasswordSalt { get; init; }
}