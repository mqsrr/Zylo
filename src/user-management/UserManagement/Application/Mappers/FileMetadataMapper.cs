using UserManagement.Application.Contracts.Responses;
using UserManagement.Application.Models;

namespace UserManagement.Application.Mappers;

internal static class FileMetadataMapper
{
    internal static FileMetadataResponse ToResponse(this FileMetadata metadata)
    {
        return new FileMetadataResponse
        {
            Url = metadata.AccessUrl.Url,
            ContentType = metadata.ContentType,
            FileName = metadata.FileName
        };
    }
}