using Newtonsoft.Json;

namespace NotificationService.Models;

public record struct UserId(Guid Value)
{
    public static UserId Parse(string uid)
    {
        return new UserId(Guid.Parse(uid));
    }
}

public sealed class UserIdConverter: JsonConverter<UserId>
{

    public override void WriteJson(JsonWriter writer, UserId value, JsonSerializer serializer)
    {
        writer.WriteValue(value.ToString());
    }

    public override UserId ReadJson(JsonReader reader, Type objectType, UserId existingValue, bool hasExistingValue, JsonSerializer serializer)
    {
        return UserId.Parse((string)reader.Value!);
    }
}


public sealed class User
{
    public required UserId Id { get; init; }

    public required string Email { get; init; }

    public required string EmailIv { get; init; }
}