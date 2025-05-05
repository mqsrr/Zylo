using UserManagement.Application.Contracts.Responses.Auth;
using UserManagement.Domain.Auth;

namespace UserManagement.Application.Mappers;

internal static class FileMetadataMapper
{
    public static FileMetadataResponse ToResponse(this FileMetadata metadata)
    {
        return new FileMetadataResponse
        {
            Url = metadata.AccessUrl.Url,
            ContentType = metadata.ContentType,
            FileName = metadata.FileName
        };
    }
}