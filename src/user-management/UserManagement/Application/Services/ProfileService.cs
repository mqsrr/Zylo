using Grpc.Core;
using GrpcServices;
using UserManagement.Application.Models;
using UserManagement.Application.Services.Abstractions;

namespace UserManagement.Application.Services;

internal sealed class ProfileService : UserProfileService.UserProfileServiceBase
{
    private readonly IImageService _imageService;

    public ProfileService(IImageService imageService)
    {
        _imageService = imageService;
    }

    public override async Task<UserProfileResponse> GetProfilePicture(UserProfileRequest request, ServerCallContext context)
    {
        var id = UserId.Parse(request.UserId);
        var profileImage = await _imageService.GetImageAsync(id, ImageCategory.Profile, context.CancellationToken);
        
        return new UserProfileResponse
        {
            ProfilePictureUrl = profileImage.AccessUrl.Url,
            ContentType = profileImage.ContentType,
            FileName = profileImage.FileName,
            ExpiresIn = new DateTimeOffset(profileImage.AccessUrl.ExpiresIn).ToUnixTimeMilliseconds()
        };
    }
}