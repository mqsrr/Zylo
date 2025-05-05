using UserManagement.Domain.Users;
using UserProfileService;

namespace UserManagement.Infrastructure.Mappers;

public static class GrpcUserMapper
{
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
}