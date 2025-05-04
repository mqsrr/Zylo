using NotificationService.Messages.User;
using NotificationService.Models;

namespace NotificationService.Mappers;

internal static class UserMapper
{
    public static User ToUser(this UserCreated message)
    {
        return new User
        {
            Id = message.Id,
            Email = message.Email,
            EmailIv = message.EmailIv,
        };
    }
}