using Newtonsoft.Json;
using NotificationService.Domain.Users;

namespace NotificationService.Application.Converters;

internal sealed class UserIdConverter: JsonConverter<UserId>
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