using Riok.Mapperly.Abstractions;
using UserManagement.Application.Contracts.Requests.Auth;
using UserManagement.Application.Contracts.Responses;
using UserManagement.Application.Models;

namespace UserManagement.Application.Mappers;

[Mapper]
internal static partial class UserMapper
{
    [MapperIgnoreTarget(nameof(User.ProfileImage))]
    [MapperIgnoreTarget(nameof(User.BackgroundImage))]
    internal static partial User ToUser(this RegisterRequest identity);
    
    internal static partial UserResponse ToResponse(this User user);
    
    private static Ulid MapUserIdToUlid(UserId id)
    {
        return id.Value;
    }

    private static FileMetadataResponse MapFileMetadataToResponse(FileMetadata fileMetadata)
    {
        return fileMetadata.ToResponse();
    }
}