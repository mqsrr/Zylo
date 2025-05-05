using Newtonsoft.Json;
using UserManagement.Domain.Users;

namespace UserManagement.Application.Converters;

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