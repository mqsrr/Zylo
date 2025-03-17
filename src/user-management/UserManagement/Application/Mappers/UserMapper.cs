using UserManagement.Application.Contracts.Requests.Auth;
using UserManagement.Application.Contracts.Requests.Users;
using UserManagement.Application.Contracts.Responses;
using UserManagement.Application.Models;
using UserProfileService;

namespace UserManagement.Application.Mappers;

internal static class UserMapper
{
    internal static User ToUser(this RegisterRequest request)
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

    internal static User ToUser(this UpdateUserRequest request, string id)
    {
        request.Id = UserId.Parse(id);
        return ToUser(request);
    }

    internal static GrpcUserResponse ToGrpcResponse(this User user)
    {
        return new GrpcUserResponse
        {
            Id = user.Id.ToString(),
            ProfileImage = new UserImage
            {
                Url = user.ProfileImage!.AccessUrl.Url,
                ContentType = user.ProfileImage.ContentType,
                FileName = user.ProfileImage.FileName,
            },
            BackgroundImage = new UserImage
            {
                Url = user.BackgroundImage!.AccessUrl.Url,
                ContentType = user.BackgroundImage.ContentType,
                FileName = user.BackgroundImage.FileName
            },
            Name = user.Name,
            Username = user.Username,
            Birthdate = user.BirthDate.ToString("O"),
            Bio = user.Bio,
            Location = user.Location
        };
    }

    internal static GrpcUserPreview ToGrpcResponse(this UserSummary user)
    {
        return new GrpcUserPreview
        {
            Id = user.Id.ToString(),
            Name = user.Name,
            ProfileImage = new UserImage
            {
                Url = user.ProfileImage!.AccessUrl.Url,
                ContentType = user.ProfileImage.ContentType,
                FileName = user.ProfileImage.FileName
            },
        };
    }

    internal static UserResponse ToResponse(this User user)
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