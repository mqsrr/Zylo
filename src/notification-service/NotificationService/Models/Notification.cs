using Newtonsoft.Json;

namespace NotificationService.Models;


public record struct NotificationId(Guid Value)
{
    public static NotificationId Parse(string uid)
    {
        return new NotificationId(Guid.Parse(uid));
    }

    public override string ToString()
    {
        return Value.ToString();
    }
}

public sealed class NotificationIdConverter: JsonConverter<NotificationId>
{

    public override void WriteJson(JsonWriter writer, NotificationId value, JsonSerializer serializer)
    {
        writer.WriteValue(value.ToString());
    }

    public override NotificationId ReadJson(JsonReader reader, Type objectType, NotificationId existingValue, bool hasExistingValue, JsonSerializer serializer)
    {
        return NotificationId.Parse((string)reader.Value!);
    }
}

public sealed class Notification
{
    public required NotificationId Id { get; init; }

    public required string Message { get; init; }

    public required bool IsSeen { get; init; }
}