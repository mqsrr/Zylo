using UserManagement.Application.Contracts.Requests.Auth;
using UserManagement.Application.Contracts.Requests.Users;
using UserManagement.Application.Contracts.Responses.Users;
using UserManagement.Domain.Users;

namespace UserManagement.Application.Mappers;

public static class UserMapper
{
    public static User ToUser(this RegisterRequest request)
    {
        return new User
        {
            Id = UserId.Parse(request.Id),
            ProfileImage = null,
            BackgroundImage = null,
            Name = request.Name,
            Username = request.Username,
            Bio = request.Bio,
            Location = request.Location,
            BirthDate = request.BirthDate
        };
    }

    private static User ToUser(this UpdateUserRequest request)
    {
        return new User
        {
            Id = request.Id,
            ProfileImage = null,
            BackgroundImage = null,
            Name = request.Name,
            Username = null,
            Bio = request.Bio,
            Location = request.Location,
            BirthDate = request.BirthDate
        };
    }

    public static User ToUser(this UpdateUserRequest request, string id)
    {
        request.Id = UserId.Parse(id);
        return ToUser(request);
    }

    public static UserResponse ToResponse(this User user)
    {
        return new UserResponse
        {
            Id = user.Id.Value,
            ProfileImage = user.ProfileImage!.ToResponse(),
            BackgroundImage = user.BackgroundImage!.ToResponse(),
            Name = user.Name,
            Username = user.Username!,
            BirthDate = user.BirthDate
        };
    }
}